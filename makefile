target/release/bfcr: src/bin/bfcr.rs Cargo.toml
	cargo build

bfcr: src/bin/bfcr.rs
	rustc -O $<

install:
	cargo install --path .
