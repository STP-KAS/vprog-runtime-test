# vprog runtime test

> **Experimental. Not advice.**

A host check of the kaspanet/vprogs runtime-processor guest body. [DISCLAIMER.md](DISCLAIMER.md).

## What

`run_transfer` builds one v1 input blob and calls `process_transaction`. The closure is `runtime::run` with `ExampleDepositPolicy`. That is the closure `zk/backend/risc0/runtime-processor/src/main.rs` passes in.

Two existing users, both unlocked. The lower resource id pays the higher one. A short source comes back as the guest error discriminant. The same index does too.

This is not a proof. The ELF is not executed. `node/vm` `process_transaction` on master is still `todo!`, so the node does not run this path.

The 9 Oct 2026 workshop lab is [STP-KAS/vprog-sovereign](https://github.com/STP-KAS/vprog-sovereign). This checkout stays on `f9b84a8`. [WORKSHOP.md](WORKSHOP.md).

[STP-KAS/quiet-lane](https://github.com/STP-KAS/quiet-lane) is a separate model of the settler wake. It does not call this guest.

## Test

Repository: https://github.com/STP-KAS/vprog-runtime-test

```
git clone https://github.com/STP-KAS/vprog-runtime-test.git
cd vprog-runtime-test
cargo test -p vprog-runtime-test --manifest-path vprogs/Cargo.toml
```

A pass is `3 passed`. Balances 50 and 7, amount 20, land at 30 and 27. Amount 51 is a guest error. Two indexes that are both 0 are a guest error. The first run needs a network so cargo can fetch the locked rusty-kaspa commit. The command submits nothing.

`vprogs/` is a checkout of kaspanet/vprogs `f9b84a863a7c7c20586a9cf947550475e894f72e`, under the ISC license in `vprogs/LICENSE`. One member was added: `examples/host-check`, plus that members line and the lock entries it needs. The rest of that tree is the upstream checkout. Upstream does not track this repository. On 1 Oct 2026 the command above passed those 3 tests.

## Inconsistencies

| Where | What the files say |
| --- | --- |
| Node VM | `node/vm` `process_transaction` is `todo!` on `f9b84a8`. This crate calls the ABI function of the same name, which is the guest entry, not the node VM. |
| ELF | The committed `program.elf` is what the RISC0 executor runs. This crate runs the same Rust body on the host and does not load that ELF. |
| Hasher | The ELF entry names `vprogs_zk_backend_risc0_api::Sha256`. With the `guest` feature that is the RISC0 precompile. This host check calls `process_transaction` with `vprogs_core_hashing::Sha256`, which is the type that api crate exports when `guest` is off. |
| Proof | Nothing here is proved. |

Intentions are good; thought process is questionable. STP remains delusional. Si vis pacem, para bellum.
