use std::ffi::CStr;
use std::os::raw::c_char;

/// Helper function to convert [c_char; SIZE] to string
pub(crate) fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string.to_str().expect("Failed to convert vulkan raw string.").to_owned()
}

pub(crate) unsafe fn struct_as_bytes<T>(s: &T) -> &[u8] {
    std::slice::from_raw_parts(
        (s as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}