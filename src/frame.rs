use crate::client;
use std::{
    error::Error,
    ffi::{CStr, CString},
    io,
    os::fd::RawFd,
    path::Path,
    ptr, slice,
};
use videostream_sys as ffi;

/// The Frame structure handles the frame and underlying framebuffer.  A frame
/// can be an image or a single video frame, the distinction is not considered.
///
/// A frame can be created and used as a free-standing frame, which means it is
/// not published through a Host nor was it created from a receiving Client. A
/// free-standing frame can be mapped and copied to other frames which provides
/// an optimized method for resizing or converting between formats.
pub struct Frame {
    ptr: *mut ffi::VSLFrame,
}

unsafe impl Send for Frame {}

impl Frame {
    pub fn new(
        width: u32,
        height: u32,
        stride: u32,
        fourcc_str: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let buf = fourcc_str.as_bytes();
        if buf.len() != 4 {
            return Err("fourcc must be 4 character ascii code".into());
        }
        let mut fourcc: u32 = 0;
        for i in 0..buf.len() {
            fourcc += (buf[i] as u32) << i * 8;
        }

        let ptr = unsafe {
            ffi::vsl_frame_init(width, height, stride, fourcc, std::ptr::null_mut(), None)
        };

        if ptr.is_null() {
            let err = io::Error::last_os_error();
            return Err(Box::new(err));
        }
        return Ok(Frame { ptr });
    }

    pub fn alloc(&self, path: Option<&Path>) -> Result<(), Box<dyn Error>> {
        let path_ptr;
        if let Some(path) = path {
            let path = path.to_str().unwrap();
            let path = CString::new(path).unwrap();
            path_ptr = path.into_raw();
        } else {
            path_ptr = ptr::null_mut();
        }
        let ret = unsafe { ffi::vsl_frame_alloc(self.ptr, path_ptr) } as i32;
        if ret != 0 {
            let err = io::Error::last_os_error();
            return Err(Box::new(err));
        }
        return Ok(());
    }

    pub fn wrap(ptr: *mut ffi::VSLFrame) -> Result<Self, ()> {
        if ptr.is_null() {
            return Err(());
        }

        return Ok(Frame { ptr });
    }

    pub fn release(&self) {
        unsafe { ffi::vsl_frame_release(self.ptr) };
    }

    pub fn wait(client: &client::Client, until: i64) -> Result<Self, Box<dyn Error>> {
        let wrapper = client.get_frame(until)?;
        return Ok(Frame { ptr: wrapper.ptr });
    }

    pub fn trylock(&self) -> Result<(), Box<dyn Error>> {
        let ret = unsafe { ffi::vsl_frame_trylock(self.ptr) };
        if ret != 0 {
            let err = io::Error::last_os_error();
            return Err(Box::new(err));
        }
        return Ok(());
    }

    pub fn unlock(&self) -> Result<(), Box<dyn Error>> {
        if unsafe { ffi::vsl_frame_unlock(self.ptr) as i32 } == -1 {
            let err = io::Error::last_os_error();
            return Err(Box::new(err));
        }
        return Ok(());
    }

    pub fn serial(&self) -> i64 {
        return unsafe { ffi::vsl_frame_serial(self.ptr) };
    }

    pub fn timestamp(&self) -> i64 {
        let timestamp: i64 = unsafe { ffi::vsl_frame_timestamp(self.ptr) };
        return timestamp;
    }

    pub fn duration(&self) -> i64 {
        return unsafe { ffi::vsl_frame_duration(self.ptr) };
    }

    pub fn pts(&self) -> i64 {
        return unsafe { ffi::vsl_frame_pts(self.ptr) };
    }

    pub fn dts(&self) -> i64 {
        return unsafe { ffi::vsl_frame_dts(self.ptr) };
    }

    pub fn expires(&self) -> i64 {
        return unsafe { ffi::vsl_frame_expires(self.ptr) };
    }

    pub fn fourcc(&self) -> u32 {
        return unsafe { ffi::vsl_frame_fourcc(self.ptr) };
    }

    pub fn width(&self) -> i32 {
        let width: std::os::raw::c_int = unsafe { ffi::vsl_frame_width(self.ptr) };
        return width as i32;
    }

    pub fn height(&self) -> i32 {
        let height: std::os::raw::c_int = unsafe { ffi::vsl_frame_height(self.ptr) };
        return height as i32;
    }

    pub fn size(&self) -> i32 {
        return unsafe { ffi::vsl_frame_size(self.ptr) as i32 }; //Needs work
    }

    /*
    pub fn stride(&self) -> i32 {
        return unsafe { ffi::vsl_frame_stride(self.ptr) as i32};
    }
    */

    pub fn handle(&self) -> Option<i32> {
        let handle: std::os::raw::c_int = unsafe { ffi::vsl_frame_handle(self.ptr) };
        if handle == -1 {
            return None;
        }
        return Some(handle as i32);
    }

    pub fn paddr(&self) -> Option<isize> {
        let ret = unsafe { ffi::vsl_frame_paddr(self.ptr) };
        if ret == -1 {
            return None;
        }
        return Some(ret);
    }

    pub fn path(&self) -> Option<&str> {
        let ret = unsafe { ffi::vsl_frame_path(self.ptr) };
        if ret.is_null() {
            return None;
        }
        let path = unsafe {
            match CStr::from_ptr(ret).to_str() {
                Ok(path) => path,
                Err(_) => {
                    return None;
                }
            }
        };
        return Some(path);
    }

    pub fn mmap(&self) -> Result<&[u8], ()> {
        if self.handle() == None {
            return Err(());
        }
        let mut size: usize = 0;
        let ptr = unsafe { ffi::vsl_frame_mmap(self.ptr, &mut size as *mut usize) };
        if ptr.is_null() || size == 0 {
            return Err(());
        }
        return Ok(unsafe { slice::from_raw_parts(ptr as *const u8, size) });
    }

    pub fn mmap_mut(&self) -> Result<&mut [u8], ()> {
        if self.handle() == None {
            return Err(());
        }
        let mut size: usize = 0;
        let ptr = unsafe { ffi::vsl_frame_mmap(self.ptr, &mut size as *mut usize) };
        if ptr.is_null() || size == 0 {
            return Err(());
        }
        return Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size) });
    }

    pub fn munmap(&self) {
        return unsafe { ffi::vsl_frame_munmap(self.ptr) };
    }

    pub fn attach(&self, fd: RawFd, size: usize, offset: usize) -> Result<(), Box<dyn Error>> {
        let ret = unsafe { ffi::vsl_frame_attach(self.ptr, fd, size, offset) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            return Err(Box::new(err));
        }
        return Ok(());
    }

    pub fn get_ptr(&self) -> *mut ffi::VSLFrame {
        return self.ptr.clone();
    }
}

impl TryFrom<*mut ffi::VSLFrame> for Frame {
    type Error = ();

    fn try_from(ptr: *mut ffi::VSLFrame) -> Result<Self, Self::Error> {
        if ptr.is_null() {
            return Err(());
        }
        return Ok(Frame { ptr });
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            ffi::vsl_frame_unlock(self.ptr);
            ffi::vsl_frame_release(self.ptr);
        };
    }
}
