cargo install samply
cargo build
samply record ./target/debug/dumpster-engine RUST_BACKTRACE=1
exit
