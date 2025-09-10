cargo install samply
cargo build
samply record ./target/debug/examples/volleyball RUST_BACKTRACE=1
exit
