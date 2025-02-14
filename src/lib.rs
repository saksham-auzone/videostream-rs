//! DeepView VideoStream Library for Rust
//!
//! The VideoStream Library provides a mechanism for zero-copy sharing of video
//! frames across processes and containers.  The sharing is done through dmabuf
//! or shared-memory buffers with signalling over UNIX Domain Sockets.
//!
//! Au-Zone Technologies provides professional support through the
//! [`DeepView Support Portal`].
//!
//! [`DeepView Support Portal`]: https://support.deepviewml.com

use std::{error::Error, ffi::CStr, fmt};
use videostream_sys as ffi;
/// The frame module provides the common frame handling functionality.
pub mod frame;

/// The client module provides the frame subscription functionality.
pub mod client;

/// The host module provides the frame sharing functionality.
pub mod host;

pub mod encoder;

#[derive(Debug)]
struct NullStringError;

impl Error for NullStringError {}

impl fmt::Display for NullStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid null string provided")
    }
}

pub fn version() -> &'static str {
    let cstr = unsafe { CStr::from_ptr(ffi::vsl_version()) };
    return cstr.to_str().unwrap();
}

pub fn timestamp() -> i64 {
    return unsafe { ffi::vsl_timestamp() };
}

pub fn fourcc(code: &str) -> u32 {
    let bytes = code.as_bytes();
    let mut fourcc: u32 = 0;

    for i in 0..4 {
        fourcc |= (bytes[i] as u32) << (i * 8);
    }

    return fourcc;
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use videostream_sys::vsl_version;

    #[test]
    fn test_version() {
        let c_ver = unsafe { CStr::from_ptr(vsl_version()) };
        println!("VideoStream Library {}", c_ver.to_str().unwrap());
    }
}
