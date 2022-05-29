use {
    zstd_sys::{
        ZSTD_isError,
        ZSTD_createDCtx,

        ZSTD_getDecompressedSize,
        ZSTD_decompressDCtx,

        ZSTD_DCtx,

        ZSTD_freeDCtx
    },

    std::{
        ffi::c_void,
        num::NonZeroUsize,

        intrinsics::unlikely
    },

    tokio::task::spawn_blocking,
    crate::UnsafeCell
};

pub struct ZStdDctx {
    inner: UnsafeCell<ZSTD_DCtx>,
}

impl ZStdDctx {
    pub async fn decompress_async(
        &mut self,
        src: &mut Vec<u8>
    ) -> Option<Vec<u8>> {
        unsafe {
            let inplace = std::mem::take(src);
            let inplace_capacity = inplace.capacity();

            let dctx_ptr = self.inner;
            let inplace_leak: &'static mut [u8] = inplace.leak();

            let inplace_ptr = UnsafeCell { ptr: inplace_leak.as_mut_ptr() };
            let inplace_len = inplace_leak.len();

            let result = spawn_blocking(move || {
                let dctx_ptr = dctx_ptr;
                let inplace_ptr = inplace_ptr;
                Self::decompress_dctx(
                    dctx_ptr.ptr,
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

    pub fn decompress(
        &mut self,
        src: &[u8]
    ) -> Option<Vec<u8>> {
        unsafe {
            Self::decompress_dctx(
                self.inner.ptr,
                src.as_ptr(),
                src.len()
            )
        }
    }

    pub fn decompressed_size_of(
        src: &[u8]
    ) -> Option<NonZeroUsize> {
        unsafe {
            Self::unsafe_decompressed_size_of(
                src.as_ptr(),
                src.len()
            )
        }
    }

    unsafe fn unsafe_decompressed_size_of(
        src: *const u8,
        srclen: usize
    ) -> Option<NonZeroUsize> {
        let result = ZSTD_getDecompressedSize(
            src as *const c_void,
            srclen
        ) as usize;

        if unlikely(result == 0) {
            None
        } else {
            Some(NonZeroUsize::new_unchecked(result))
        }
    }

    unsafe fn decompress_dctx(
        dctx: *mut ZSTD_DCtx,
        src: *const u8,
        srclen: usize,
    ) -> Option<Vec<u8>> {
        let size = if let Some(s) = Self::unsafe_decompressed_size_of(src, srclen) {
            s
        } else {
            return None
        }.into();

        let mut buffer = Vec::<u8>::with_capacity(size);
        buffer.set_len(size);

        let code = ZSTD_decompressDCtx(
            dctx,
            buffer.as_mut_ptr() as *mut c_void,
            size,
            src as *const c_void,
            srclen
        );

        if unlikely(ZSTD_isError(code) == 1) {
            None
        } else {
            Some(buffer)
        }
    }

    pub fn new() -> Self {
        let dctx = unsafe { ZSTD_createDCtx() };
        Self { inner: UnsafeCell { ptr: dctx } }
    }
}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeDCtx(self.inner.ptr);
        }
    }
}
