use crate::alloc::{GlobalAlloc, Layout, System};
use crate::sys::pal::c;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { c::malloc(layout.size() as core::ffi::c_ulong) as *mut u8 }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { c::free(ptr as *mut core::ffi::c_void) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            c::realloc(ptr as *mut core::ffi::c_void, new_size as core::ffi::c_ulong) as *mut u8
        }
    }
}