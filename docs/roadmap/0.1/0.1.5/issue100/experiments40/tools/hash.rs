// Diagnostic FFI only: callers own a readable input and a 32-byte output.
#[no_mangle]
pub unsafe extern "C" fn diagnostic_blake3(input: *const u8, length: usize, output: *mut u8) {
    let bytes = if length == 0 { &[] } else { std::slice::from_raw_parts(input, length) };
    std::ptr::copy_nonoverlapping(blake3::hash(bytes).as_bytes().as_ptr(), output, 32);
}
