// Diagnostic FFI: cached product scanner, no replacement CDC implementation.
#[no_mangle]
pub unsafe extern "C" fn diagnostic_cdc(input: *const u8, length: usize, lengths: *mut usize, capacity: usize) -> usize {
    let bytes = if length == 0 { &[] } else { std::slice::from_raw_parts(input, length) };
    let mut count = 0usize;
    let result = layerfs_content::file::cdc::FastCdc::new().scan(std::io::Cursor::new(bytes), |chunk| {
        if count < capacity { *lengths.add(count) = chunk.len(); }
        count += 1;
        Ok(())
    });
    if result.is_err() { usize::MAX } else { count }
}
