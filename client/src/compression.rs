#[derive(Debug, Clone)]
pub struct Compression {
    pub threshold: usize,

    pub level: i32,
    pub profit: f32,
}
