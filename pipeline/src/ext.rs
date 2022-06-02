use {
    tokio::{
        io::{BufReader, BufWriter},
        net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}
    },

    serde::Deserialize
};

pub type Reader = BufReader<OwnedReadHalf>;
pub type Writer = BufWriter<OwnedWriteHalf>;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct BufferSize {
    pub read: usize,
    pub write: usize
}

impl IntoBuffered for TcpStream {
    type Bufferized = (Reader, Writer);

    fn into_buffered_with_capacity(
        self,
        capacity: &BufferSize
    ) -> Self::Bufferized {
        let (r, w) = self.into_split();

        (
            Reader::with_capacity(capacity.read, r),
            Writer::with_capacity(capacity.write, w)
        )
    }
}

pub trait IntoBuffered {
    type Bufferized;

    fn into_buffered_with_capacity(self, capacity: &BufferSize) -> Self::Bufferized;

    fn into_buffered(self) -> Self::Bufferized
    where Self: Sized
    {
        self.into_buffered_with_capacity(&BufferSize::default())
    }
}

impl Default for BufferSize {
    fn default() -> Self {
        Self::all(4096)
    }
}

impl BufferSize {    
    pub fn all(capacity: usize) -> Self {
        Self { read: capacity
             , write: capacity }
    }

    pub fn new(read: usize, write: usize) -> Self {
        Self { read
             , write }
    }
}
