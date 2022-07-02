use std::ffi::CString;

pub fn i8_slice_to_cstring(input: &[i8]) -> CString {
    let mut result_vec = Vec::with_capacity(input.len());
    for byte in input {
        if *byte != 0 {
            result_vec.push(*byte as u8)
        } else {
            break;
        }
    }
    unsafe { CString::from_vec_unchecked(result_vec) }
}
