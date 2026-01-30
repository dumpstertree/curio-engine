cargo install samply
cargo build --release
samply record ./target/debug/dumpster-engine RUST_BACKTRACE=1
exit
