# BFCR

BFCR is a brainfuck compiler in Rust. It uses Rust as the target language, so it compiles/transpiles brainfuck programs into Rust.

### Build

```sh
$ cd bfcr
$ cargo build
```

### Usage

```sh
$ ./bfcr FILE.bf
```

This will produce a file named `FILE.rs` and an executable named `FILE` (on a Unix-like system).

The initial number of cells is the same as the total number of `>` in the source file. If not enough, more cells will be automatically allocated at the run-time. The initial number of cells can also be set with `--initial-cells`. The maximum number of cells is the same as `usize::MAX`.

Having unmatched `[` / `]` in the source code would be a compile-time error, while going beyond the left bound (to the left of the initial cell) would be a run-time error.