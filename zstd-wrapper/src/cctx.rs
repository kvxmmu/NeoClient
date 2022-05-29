use {
    zstd_sys::{
        ZSTD_createCCtx,
        ZSTD_compressCCtx,
        ZSTD_CCtx,

        ZSTD_isError,

        ZSTD_freeCCtx
    },

    std::{
        ffi::c_void,

        intrinsics::unlikely
    },

    tokio::task::spawn_blocking,
    crate::UnsafeCell
};

pub struct ZStdCctx {
    inner: UnsafeCell<ZSTD_CCtx>,
    
    level: i32,
    profit: f32,
}

impl ZStdCctx {
    pub async fn compress_async(
        &mut self,
        src: &mut Vec<u8>
    ) -> Option<Vec<u8>> {
        unsafe {
            let cctx_ptr = self.inner;
            
            let inplace = std::mem::take(src);
            let inplace_capacity = inplace.capacity();
            let inplace_leak: &'static mut [u8] = inplace.leak();

            let inplace_ptr = UnsafeCell { ptr: inplace_leak.as_mut_ptr() };
            let inplace_len = inplace_leak.len();

            let level = self.level;
            let profit = self.profit;

            let result = spawn_blocking(move || {
                let inplace_ptr = inplace_ptr;
                let cctx_ptr = cctx_ptr;

                Self::compress_cctx(
                    cctx_ptr.ptr,
                    level,
                    profit,
                    inplace_ptr.ptr,
                    inplace_len
                )
            }).await;

            *src = Vec::from_raw_parts(
                inplace_ptr.ptr,
                inplace_len,
                inplace_capacity
            );

            if let Ok(result) = result {
                result
            } else {
                None
            }
        }
    }

    pub fn compress(
        &mut self,
        src: &[u8]
    ) -> Option<Vec<u8>> {
        unsafe {
            Self::compress_cctx(
                self.inner.ptr,
                self.level,
                self.profit,
                src.as_ptr(),
                src.len()
            )
        }
    }

    unsafe fn compress_cctx(
        cctx: *mut ZSTD_CCtx,
        level: i32,
        profit: f32,
        src: *const u8,
        srclen: usize,
    ) -> Option<Vec<u8>> {
        let mut dst = Vec::with_capacity(srclen);
        dst.set_len(srclen);

        let compressed_size = ZSTD_compressCCtx(
            cctx,
            dst.as_mut_ptr() as *mut c_void,
            dst.len(),
            src as *const c_void,
            srclen,
            level
        );

        if unlikely(ZSTD_isError(compressed_size) == 1) {
            None
        } else if Self::inner_enough_profit(profit, srclen, compressed_size) {
            dst.set_len(compressed_size);
            Some(dst)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn enough_profit(
        &self,
        was: usize,
        now: usize
    ) -> bool {
        Self::inner_enough_profit(self.profit, was, now)
    }

    #[inline(always)]
    fn inner_enough_profit(
        profit: f32,
        was: usize,
        now: usize
    ) -> bool {
        if now >= was {
            false
        } else {
            let diff = (was - now) as f32;

            diff / was as f32 >= profit
        }
    }

    pub fn new(
        level: i32,
        profit: f32,
    ) -> Self {
        let dctx = unsafe { ZSTD_createCCtx() };
        Self { inner: UnsafeCell { ptr: dctx }
             , level
             , profit: profit / 100.0 }
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeCCtx(self.inner.ptr);
        }
    }
}
