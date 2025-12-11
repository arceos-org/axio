mod impls;

/// A trait describing a generic byte buffer for I/O operations.
///
/// This trait serves as an optional extension for types implementing
/// `Read` or `Write`. A reader or writer may not have a deterministic length,
/// but an `IoBuf` does.
pub trait IoBuf {
    /// Returns the number of bytes between the current position and the end of
    /// the buffer.
    fn remaining(&self) -> usize;
}
