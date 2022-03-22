use {
    zstd_sys::{
        ZSTD_decompressDCtx,
        ZSTD_compressCCtx,
        ZSTD_isError,
        
        ZSTD_CCtx,
        ZSTD_DCtx,

        ZSTD_createDCtx,
        ZSTD_createCCtx
    },

    std::{
        intrinsics::unlikely,
        thread_local,
    },

    once_cell::sync::Lazy
};

#[thread_local]
static mut THREAD_CCTX: Lazy<Box<ZSTD_CCtx>> = Lazy::new(|| unsafe { Box::from_raw(ZSTD_createCCtx()) });

#[thread_local]
static mut THREAD_DCTX: Lazy<Box<ZSTD_DCtx>> = Lazy::new(|| unsafe { Box::from_raw(ZSTD_createDCtx()) });

pub fn decompress(
    src: Vec<u8>,
    max_buf: usize,
) -> Option<(Vec<u8>, usize)> {
    let mut decompressed = vec![0; max_buf];
    let new_size;

    let dctx_ptr = unsafe {
        THREAD_DCTX.as_mut() as *mut ZSTD_DCtx
    };

    unsafe {
        new_size = ZSTD_decompressDCtx(
            dctx_ptr,
            decompressed.as_mut_ptr() as *mut std::ffi::c_void,
            max_buf,

            src.as_ptr() as *const std::ffi::c_void,
            src.len()
        );

        if unlikely(ZSTD_isError(new_size) == 1) {
            return None;
        }
    }

    Some(
        (decompressed, new_size)
    )
}

pub fn compress(
    src: &[u8],
    min_profit: f32,
    level: i32,
) -> Option<(Vec<u8>, usize)> {
    let length = src.len();
    let mut buf = vec![0; length];
    let new_size;
    let percents;

    let cctx = unsafe {
        THREAD_CCTX.as_mut() as *mut ZSTD_CCtx
    };

    unsafe {
        new_size = ZSTD_compressCCtx(
            cctx,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            length,

            src.as_ptr() as *const std::ffi::c_void,
            src.len(),

            level
        );

        if unlikely((ZSTD_isError(new_size) == 1) || (new_size == length)) {
            return None;
        } else if new_size < length {
            percents = (new_size as f32) * 100.0 / (length as f32);

            if min_profit > (100.0 - percents) {
                return None;  // Compression will not make sense
            }
        } else {
            return None;
        }
    }

    log::debug!("{} compressed to {} ({}% from original size)", length, new_size, percents as u64);

    Some(
        (buf, new_size)
    )
}