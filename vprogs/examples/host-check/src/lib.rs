//! Host check of the runtime-processor guest body.
//!
//! [`run_transfer`] calls [`process_transaction`] with [`runtime::run`] and
//! [`ExampleDepositPolicy`]. That is the closure `main` passes to the ABI.
//! The check does not execute the ELF and does not prove.

use std::mem::size_of;

use vprogs_core_codec::Writer;
use vprogs_core_types::{AccessMetadata, AccessType, ResourceId};
use vprogs_l1_utils::tx_id_v1;
use vprogs_zk_abi::transaction_processor::process_transaction;
use vprogs_zk_abi::Read;
use vprogs_zk_backend_risc0_runtime_processor::deposit_policy::ExampleDepositPolicy;
use vprogs_zk_backend_risc0_runtime_processor::lock::LockEnum;
use vprogs_zk_backend_risc0_runtime_processor::lock_variants::UnlockedLockView;
use vprogs_zk_backend_risc0_runtime_processor::runtime;
use vprogs_zk_backend_risc0_runtime_processor::user::{user_total_len, write_user};

/// Transfer action tag in the guest instruction stream.
const ACTION_TRANSFER: u8 = 0x03;

/// One unlocked user in access-metadata order.
#[derive(Clone, Copy)]
struct Seat {
    id: [u8; 32],
    balance: u64,
}

/// Bytes the guest wrote back to the host.
pub struct GuestOut {
    /// `0x00` on success, `0x01` on a guest error.
    pub discriminant: u8,
    /// Balances in access-metadata order, present only on success.
    pub balances: Option<(u64, u64)>,
}

struct MemHost {
    input: Vec<u8>,
    output: Vec<u8>,
}

impl Read for MemHost {
    fn read_blob(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.input)
    }
}

impl Writer for MemHost {
    fn write(&mut self, buf: &[u8]) {
        self.output.extend_from_slice(buf);
    }
}

/// Runs one transfer of `amount` from the lower id to the higher id.
///
/// Both seats are existing unlocked users. The lower id is access index 0.
pub fn run_transfer(source_balance: u64, dest_balance: u64, amount: u64) -> GuestOut {
    let source = Seat { id: [1u8; 32], balance: source_balance };
    let dest = Seat { id: [2u8; 32], balance: dest_balance };
    execute(source, dest, 0, 1, amount)
}

/// Runs a transfer whose source index and dest index are both 0.
pub fn run_same_index(balance: u64, amount: u64) -> GuestOut {
    let source = Seat { id: [1u8; 32], balance };
    let dest = Seat { id: [2u8; 32], balance };
    execute(source, dest, 0, 0, amount)
}

fn execute(source: Seat, dest: Seat, source_idx: u8, dest_idx: u8, amount: u64) -> GuestOut {
    let wire = encode_inputs(source, dest, source_idx, dest_idx, amount);
    let mut host = MemHost { input: wire, output: Vec::new() };
    let mut journal = Vec::new();
    process_transaction::<vprogs_core_hashing::Sha256>(
        &mut host,
        &mut journal,
        |tx, _merge_idx, _context_hash, resources, exits, deposit| {
            runtime::run(tx, resources, exits, deposit, &ExampleDepositPolicy)
        },
    );
    assert!(!journal.is_empty(), "the guest writes an output commitment");
    decode_out(&host.output)
}

fn encode_inputs(
    source: Seat,
    dest: Seat,
    source_idx: u8,
    dest_idx: u8,
    amount: u64,
) -> Vec<u8> {
    let payload = payload(source, dest, source_idx, dest_idx, amount);
    let rest = b"rest";
    let tx_id = tx_id_v1(&payload, rest);

    let mut tx = Vec::new();
    push_blob(&mut tx, &payload);
    push_blob(&mut tx, rest);

    let mut exec = Vec::new();
    exec.extend_from_slice(&[0u8; 32]);
    push_blob(&mut exec, &tx);
    push_resource(&mut exec, 0, &user_bytes(source.balance));
    push_resource(&mut exec, 1, &user_bytes(dest.balance));

    let mut inputs = Vec::new();
    inputs.extend_from_slice(&1u16.to_le_bytes());
    inputs.extend_from_slice(&tx_id);
    inputs.extend_from_slice(&0u32.to_le_bytes());
    inputs.extend_from_slice(&exec);
    inputs
}

fn payload(source: Seat, dest: Seat, source_idx: u8, dest_idx: u8, amount: u64) -> Vec<u8> {
    let mut ix = Vec::new();
    ix.extend_from_slice(&0u32.to_le_bytes());
    ix.extend_from_slice(&1u32.to_le_bytes());
    ix.push(ACTION_TRANSFER);
    ix.push(source_idx);
    ix.push(dest_idx);
    ix.extend_from_slice(&amount.to_le_bytes());
    ix.push(0);

    let mut body = Vec::new();
    body.extend_from_slice(&2u32.to_le_bytes());
    push_meta(&mut body, source.id);
    push_meta(&mut body, dest.id);
    body.extend_from_slice(&ix);
    body
}

fn push_meta(buf: &mut Vec<u8>, id: [u8; 32]) {
    let meta = AccessMetadata::write(ResourceId::from(id));
    assert_eq!(meta.access_type, AccessType::Write);
    let n = size_of::<AccessMetadata>();
    let ptr = &meta as *const AccessMetadata as *const u8;
    // `AccessMetadata` is `Unaligned` and fully initialized. The guest reads this same width.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, n) };
    buf.extend_from_slice(bytes);
}

fn user_bytes(balance: u64) -> Vec<u8> {
    let lock = LockEnum::Unlocked(UnlockedLockView);
    let mut buf = vec![0u8; user_total_len(&lock)];
    write_user(&mut buf, balance, &[0u8; 32], &lock).expect("user wire");
    buf
}

fn push_resource(buf: &mut Vec<u8>, index: u32, data: &[u8]) {
    buf.extend_from_slice(&index.to_le_bytes());
    push_blob(buf, data);
}

fn push_blob(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(bytes);
}

fn decode_out(mut buf: &[u8]) -> GuestOut {
    let discriminant = buf[0];
    buf = &buf[1..];
    if discriminant != 0 {
        return GuestOut { discriminant, balances: None };
    }
    let first = read_user_balance(&mut buf);
    let second = read_user_balance(&mut buf);
    assert!(buf.is_empty(), "guest output has trailing bytes");
    GuestOut { discriminant, balances: Some((first, second)) }
}

fn read_user_balance(buf: &mut &[u8]) -> u64 {
    assert_eq!(buf[0], 1, "a transfer marks both users dirty");
    *buf = &buf[1..];
    let data = read_blob(buf);
    u64::from_le_bytes(data[1..9].try_into().expect("balance"))
}

fn read_blob(buf: &mut &[u8]) -> Vec<u8> {
    let len = u32::from_le_bytes(buf[..4].try_into().expect("blob len")) as usize;
    *buf = &buf[4..];
    let data = buf[..len].to_vec();
    *buf = &buf[len..];
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_transfer_moves_both_balances() {
        let out = run_transfer(50, 7, 20);
        assert_eq!(out.discriminant, 0);
        assert_eq!(out.balances, Some((30, 27)));
    }

    #[test]
    fn a_short_source_is_a_guest_error() {
        let out = run_transfer(50, 7, 51);
        assert_eq!(out.discriminant, 1);
        assert_eq!(out.balances, None);
    }

    #[test]
    fn the_same_index_is_a_guest_error() {
        let out = run_same_index(50, 1);
        assert_eq!(out.discriminant, 1);
        assert_eq!(out.balances, None);
    }
}
