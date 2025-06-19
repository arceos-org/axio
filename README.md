# axio

---

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/axio)](https://crates.io/crates/axio)
[![Docs.rs](https://docs.rs/axio/badge.svg)](https://docs.rs/axio)
[![CI](https://github.com/arceos-org/axio/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/arceos-org/axio/actions/workflows/ci.yml)
[![DeepWiki](https://img.shields.io/badge/DeepWiki-docs-8A2BE2)](https://deepwiki.org/arceos-org/axio)

</div>

---

[`std::io`][1]-like I/O traits for `no_std` environment.

[1]: https://doc.rust-lang.org/std/io/index.html

## Example

```rust
use axio::{BufReader, Read};
use std::io::Write as StdWrite; // 导入标准库的Write trait

fn main() {
    let data = b"hello world";
    let mut reader = BufReader::new(&data[..]);

    let mut buf = [0u8; 5];
    reader.read_exact(&mut buf).expect("读取失败");
    println!("读取前5字节: {:?}", String::from_utf8_lossy(&buf));

    let mut writer = Vec::new();
    writer.write_all(b"test").expect("写入失败");
    println!("写入内容: {:?}", writer);
}
```
