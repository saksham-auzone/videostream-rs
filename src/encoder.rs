use crate::{frame, NullStringError};
use std::{error::Error, os::raw::c_int};
use videostream_sys as ffi;

pub struct Encoder {
    ptr: *mut ffi::VSLEncoder,
}

pub struct VSLEncoderProfile {
    profile: ffi::VSLEncoderProfile,
}

pub struct VSLRect {
    rect: ffi::vsl_rect,
}

impl VSLRect {
    pub fn new(x: c_int, y: c_int, width: c_int, height: c_int) -> Self {
        return VSLRect {
            rect: ffi::vsl_rect {
                x,
                y,
                width,
                height,
            },
        };
    }

    pub fn get_width(&self) -> c_int {
        return (self.rect).width;
    }

    pub fn get_height(&self) -> c_int {
        // println!("{}",height);
        return (self.rect).height;
    }

    pub fn get_x(&self) -> c_int {
        return (self.rect).x;
    }

    pub fn get_y(&self) -> c_int {
        return (self.rect).y;
    }
}

impl Encoder {
    pub fn create(profile: u32, output_fourcc: u32, fps: c_int) -> Self {
        return Encoder {
            ptr: unsafe { ffi::vsl_encoder_create(profile, output_fourcc, fps) },
        };
    }

    pub fn new_output_frame(
        &self,
        width: c_int,
        height: c_int,
        duration: i64,
        pts: i64,
        dts: i64,
    ) -> Result<frame::Frame, Box<dyn Error>> {
        let frame_ptr = unsafe {
            ffi::vsl_encoder_new_output_frame(self.ptr, width, height, duration, pts, dts)
        };
        if frame_ptr.is_null() {
            return Err(Box::new(NullStringError {}));
        }
        match frame_ptr.try_into() {
            Ok(frame) => return Ok(frame),
            Err(()) => return Err(Box::new(NullStringError {})),
        };
    }

    pub fn frame(
        &self,
        source: &frame::Frame,
        destination: &frame::Frame,
        crop_region: &mut VSLRect,
        keyframe: *mut c_int,
    ) -> i32 {
        return unsafe {
            ffi::vsl_encode_frame(
                self.ptr,
                source.get_ptr(),
                destination.get_ptr(),
                &mut crop_region.rect,
                keyframe,
            )
        };
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { ffi::vsl_encoder_release(self.ptr) }
    }
}
