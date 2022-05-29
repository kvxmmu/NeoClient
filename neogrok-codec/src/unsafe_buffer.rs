/// To reduce overhead
/// Buffers created via this function are meant to be overwritten
pub unsafe fn create_buffer(
    length: usize
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(length);
    buf.set_len(length);

    buf
}
