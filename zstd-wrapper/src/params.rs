#[derive(Debug, Clone)]
pub struct ZStdParams {
    pub threshold: usize,

    pub level: i32,
    pub profit: f32,
}

impl ZStdParams {
    #[inline(always)]
    pub fn enough_profit(
        &self,
        was: usize,
        now: usize
    ) -> bool {
        let fraction = now as f32 / was as f32;

        fraction >= self.profit
    }
}
