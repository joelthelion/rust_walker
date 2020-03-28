Asynchronous randomized large filesystem explorer
=================================================

Quickly sample large, remote directory trees. Uses async IO for speed on high latency connections.

Example output:

```
[joel@panda rust_walker]$ rust_walker | head -n 20
./Cargo.toml
./.vimrc
./Cargo.lock
./readme.md
./.gitignore
./src/rust_walker.rs
./src/legacy.rs
./src
./target/.rustc_info.json
./target/debug/.cargo-lock
./target/rls/.rustc_info.json
./target/doc/settings.html
./.git/COMMIT_EDITMSG
./target/debug/rust_walker.d
./target/debug/rust_walker
./target/doc/favicon.ico
./.git/config
./target/rls/debug/.cargo-lock
./.git/packed-refs
./target/doc/.lock
```
