use std::os::raw::c_void;

pub struct UnsafeMappedBuffer {
    pub ptr: *mut c_void,
    pub size: usize
}

unsafe impl Send for UnsafeMappedBuffer {}
unsafe impl Sync for UnsafeMappedBuffer {}

pub struct UnsafePtr<T> {
    pub ptr: *mut T
}

impl<T> UnsafePtr<T> {
    pub unsafe fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

unsafe impl<T> Send for UnsafePtr<T> {}
unsafe impl<T> Sync for UnsafePtr<T> {}
