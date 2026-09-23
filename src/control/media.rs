use crate::{control::{IControlLoader, ControlMode, LoaderRet}, surface::Surface, media };

const MEDIA_SIZE: usize = 1024 * 8;

static mut MEDIA_PKG: [u8; MEDIA_SIZE] = [0 as u8; MEDIA_SIZE];

impl IControlLoader for Surface {
    fn process_loader_start(&mut self, mode: ControlMode, _len: usize) -> usize {
        match mode {
            ControlMode::PKGMode => unsafe {  MEDIA_PKG.len() },
            _ => 0
        }
    }

    fn process_loader_data(&mut self, buf: &[u8], pos: usize) {
        if pos < unsafe { MEDIA_PKG.len() } {
            unsafe { MEDIA_PKG[pos] = buf[0]; }
        }
    }

    fn process_loader_end(&mut self) -> LoaderRet {                
        let ret = self.media.load_pkg(unsafe { MEDIA_PKG.as_ptr() }, unsafe { MEDIA_PKG.len() }, 2);     
        match ret {
            media::Ret::Ok => LoaderRet::Ok,
            _ => LoaderRet::PKGFailed

        }        
    }
}
