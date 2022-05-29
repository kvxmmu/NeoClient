#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferSize {
    pub read: usize,
    pub write: usize
}

impl Into<BufferSize> for (usize, usize) {
    fn into(self) -> BufferSize {
        BufferSize { read: self.0
                   , write: self.1 }
    }
}
