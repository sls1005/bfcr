target/release/bfcr: src/bin/bfcr.rs Cargo.toml
	cargo build --release

bfcr: src/bin/bfcr.rs
	rustc -O $<

install:
	cargo install --path .
