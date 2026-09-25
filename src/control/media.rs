use crate::{control::{IControlLoader, ControlMode, LoaderRet}, surface::Surface, media };

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec};

#[cfg(not(feature = "alloc"))]
const MEDIA_SIZE: usize = 1024 * 8;

#[cfg(feature = "alloc")]
static mut MEDIA_PKG: Option<Box<[u8]>> = None;
#[cfg(not(feature = "alloc"))]
static mut MEDIA_PKG: [u8; MEDIA_SIZE] = [0 as u8; MEDIA_SIZE];

#[cfg(feature = "alloc")]
fn media_pkg() -> &'static mut [u8] {
    unsafe { (*core::ptr::addr_of_mut!(MEDIA_PKG)).as_deref_mut().unwrap_or(&mut []) }
}

#[cfg(not(feature = "alloc"))]
fn media_pkg() -> &'static mut [u8] {
    unsafe { &mut *core::ptr::addr_of_mut!(MEDIA_PKG) }
}

impl IControlLoader for Surface {
    #[cfg_attr(not(feature = "alloc"), allow(unused_variables))]
    fn process_loader_start(&mut self, mode: ControlMode, len: usize) -> usize {
        match mode {
            ControlMode::PKGMode => {
                #[cfg(feature = "alloc")]
                unsafe {
                    *core::ptr::addr_of_mut!(MEDIA_PKG) = Some(vec![0u8; len].into_boxed_slice());
                }
                media_pkg().len()
            },
            _ => 0
        }
    }

    fn process_loader_data(&mut self, chunk: &[u8], pos: usize) {
        let buf = media_pkg();
        if pos < buf.len() {
            buf[pos] = chunk[0];
        }
    }

    fn process_loader_end(&mut self) -> LoaderRet {                
        let buf = media_pkg();
        let ret = self.media.load_pkg(buf.as_ptr(), buf.len(), 2);     
        match ret {
            media::Ret::Ok => LoaderRet::Ok,
            _ => LoaderRet::PKGFailed

        }        
    }
}
