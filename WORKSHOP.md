# Workshop, 9 Oct 2026

Maksim Biriukov published [vprogs: a based rollup on Kaspa](https://biryukovmaxim.github.io/vprogs-workshop/). Michael Sutton pointed at that book the same day. A tweet is not a pin.

The book links kaspanet/vprogs `055ae28a` and biryukovmaxim/vprog-tictactoe `fe6b0e85`. On 9 Oct 2026 those trees were behind the live tips: `release-candidate` was `cc0d54bc` (the book tree is 10 commits behind it) and tictactoe master was `f93e52fe` (the book tree is 13 commits behind it). The tictactoe root lock names `cc0d54bc`. The guest lock still names `04cfb0ae`.

This repository does not move. `vprogs/` stays kaspanet/vprogs `f9b84a863a7c7c20586a9cf947550475e894f72e`. The host check is still `cargo test -p vprog-runtime-test --manifest-path vprogs/Cargo.toml`. It is not a RISC0 proof. `node/vm` `process_transaction` on that pin is still `todo!`.

The rules lab for the workshop shape is [STP-KAS/vprog-sovereign](https://github.com/STP-KAS/vprog-sovereign). `cargo test` there is 14 tests. `cargo run --example town` walks one village. [MAP.md](https://github.com/STP-KAS/vprog-sovereign/blob/main/MAP.md) is the use-case map, including the LUMBRIDGE proposition. That map does not change sixpack.wtf. The lab submits nothing.
