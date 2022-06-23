use std::{ffi::CStr, os::raw::c_char};

use ash::{vk, Entry};

pub unsafe fn vk_to_cstr(raw_string_array: &[c_char]) -> &CStr {
    CStr::from_ptr(raw_string_array.as_ptr())
}

pub unsafe fn get_instance_proc_addr<U>(
    entry: &Entry,
    instance: ash::vk::Instance,
    name: &CStr,
) -> Option<U> {
    let fn_ptr = entry.get_instance_proc_addr(instance, name.as_ptr());
    Some(std::mem::transmute_copy::<vk::PFN_vkVoidFunction, U>(
        &fn_ptr,
    ))
}
