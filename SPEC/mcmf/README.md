## Quick Start

First, make sure [rustup] is installed. The
[`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

To build all methods and execute the method within the zkVM, run the following
command:

```bash
cargo run -p baseline

cargo run --bin host
```



```text
.
├── baseline
│   ├── Cargo.toml
│   └── src
│       ├── lib.rs
│       └── main.rs
├── Cargo.lock
├── Cargo.toml
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── LICENSE
├── methods
│   ├── build.rs
│   ├── Cargo.toml
│   ├── guest
│   │   ├── Cargo.lock
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── main.rs
│   └── src
│       ├── lib.rs
│       └── main.rs
├── README.md
└── rust-toolchain.toml

8 directories, 17 files
```
