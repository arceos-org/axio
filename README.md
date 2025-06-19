<h1 align="center" style="margin-bottom: 0; border: none">axio</h1>

<div align="right" style="margin-bottom: 1.5em">

a [`std::io`][1]-like I/O traits for `no_std` environment.

</div>

---

<div align="center" style="
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.2em;
    margin: 0.2em 0;
">

[![Crates.io](https://img.shields.io/crates/v/axio)](https://crates.io/crates/axio)
[![Docs.rs](https://docs.rs/axio/badge.svg)](https://docs.rs/axio)
[![CI](https://github.com/arceos-org/axio/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/arceos-org/axio/actions/workflows/ci.yml)
[![DeepWiki](https://img.shields.io/badge/DeepWiki-docs-8A2BE2)](https://deepwiki.org/arceos-org/axio)
[![Dependencies](https://img.shields.io/librariesio/release/cargo/axio)](https://libraries.io/cargo/axio)
[![Downloads](https://img.shields.io/crates/d/axio)](https://crates.io/crates/axio)
[![Code Size](https://img.shields.io/github/languages/code-size/arceos-org/axio)](https://github.com/arceos-org/axio)

[![Activity](https://img.shields.io/github/commit-activity/m/arceos-org/axio)](https://github.com/arceos-org/axio/pulse)
[![Toolchain](https://img.shields.io/badge/toolchain-nightly--2025--06--18-orange)](https://rust-lang.github.io/rustup/concepts/channels.html)
[![License](https://img.shields.io/crates/l/axio)](https://github.com/arceos-org/axio/blob/main/LICENSE)

</div>

---

[1]: https://doc.rust-lang.org/std/io/index.html

## Example

```rust
fn main() {
    use axio::{Read, BufReader};
    
    let data = b"hello world";
    let mut reader = BufReader::new(&data[..]);
    let mut buf = [0u8; 5];
    reader.read_exact(&mut buf).unwrap();
}
```
