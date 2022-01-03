use {
    zstd_sys::{ZSTD_decompress, ZSTD_compress,
               ZSTD_isError}
};


pub fn decompress_data(src: &[u8]) -> ([u8; 4096], usize) {
    let mut buf: [u8; 4096] = [0; 4096];
    let length;

    unsafe {
        let src_ptr = src.as_ptr() as *const std::ffi::c_void;
        let dst_ptr = buf.as_mut_ptr() as *mut std::ffi::c_void;

        length = ZSTD_decompress(dst_ptr, 4096,
                                 src_ptr, src.len());
        if ZSTD_isError(length) == 1 {
            return (buf, 0);
        }
    };
    
    (buf, length)
}

pub fn compress_data(data: &[u8], level: i32,
                     minimal_profit: f32) -> ([u8; 4096], usize) {
    let mut buf: [u8; 4096] = [0; 4096];
    let length;

    unsafe {
        let src = data.as_ptr() as *const std::ffi::c_void;
        let dst = buf.as_mut_ptr() as *mut std::ffi::c_void;

        length = ZSTD_compress(dst, 4096,
                               src, data.len(), level);
        if (ZSTD_isError(length) == 1) || (length >= data.len()) {
            return (buf, 0);
        }

        let threshold = (data.len() as f32) / 100.0f32 * minimal_profit;
        if (threshold as usize) <= (data.len() - length) {
            return (buf, length);
        }
    };

    (buf, 0)
}
