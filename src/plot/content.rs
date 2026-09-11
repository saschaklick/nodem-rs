use core::{str, slice, fmt};

use crate::{ Size, Point, surface::{ Surface }, media::Identifier };

#[derive(Copy, Clone)]
pub struct Content {
    pub raw: *const u8,
    pub len: usize,
    #[cfg(feature = "alloc")]
    pub alloc: bool,
}

impl Default for Content {
    fn default() -> Self { 
        Content {
            raw: core::ptr::null(),
            len: 0,
            #[cfg(feature = "alloc")]
            alloc: false
        }
    }
}

impl Content {
    pub fn reset(&mut self) {
        #[cfg(feature = "alloc")]
        unsafe {                        
            if self.alloc {
                use nostd::alloc::dealloc;
                dealloc(self.raw as *mut u8, core::alloc::Layout::from_size_align(self.len, 4).unwrap());
            }
            self.alloc = false;
        }
        self.raw = core::ptr::null();
        self.len = 0;
    }
}

impl Content {
    pub fn as_str(&self) -> &str {
        return unsafe { str::from_utf8(slice::from_raw_parts(self.raw, self.len)) }.unwrap_or("");
    }

    pub fn get_size(&self, surface: &Surface) -> Size {                
        return surface.get_text_size(Identifier::Index(surface.media.typesetting.active_idx), self.as_str());
    }

    pub fn draw(&self, surface: &Surface, position: Point){        
        surface.draw_text(Identifier::Index(surface.media.typesetting.active_idx), self.as_str(), position);
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.as_str();
        let mut lines = str.lines();
        let line = lines.next().unwrap();
        write!(f, "({}/{}) \"/{}\"", line.len(), str.len(), line)?;
        if lines.count() > 1 {
            write!(f, "...")?;   
        }
        return write!(f, "");
    }
}
