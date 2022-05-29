#[derive(Debug, Clone)]
pub struct Compression {
    pub threshold: usize,
    pub profit: f32,
    pub level: u32,
}
