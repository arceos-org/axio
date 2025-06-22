//! Buffered I/O module.
//!
//! ```
//! # #![allow(unused_imports)]
//! use axio::BufReader;
//! ```

mod bufreader;

/// Re-export the `BufReader` type from the bufreader module.
pub use self::bufreader::BufReader;
