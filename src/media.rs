use cfg_block::cfg_block;
use core::{ptr, slice};

use crate::*;
use crate::stream::*;
#[cfg(feature = "ninepatch")]
use crate::surface::ninepatch::*;

pub const SYS_FONT: u8    = 0;
pub const SYS_POINTER: u8 = 252;
pub const SYS_LOGO: u8    = 253;

pub const LIBMAGIC_IMAGE:   u8 = 20;
pub const LIBMAGIC_FONT:    u8 = 30;
pub const LIBMAGIC_BORDER:  u8 = 40;
pub const LIBMAGIC_PAGE:    u8 = 50;
pub const LIBMAGIC_PROGRAM: u8 = 60;
pub const LIBMAGIC_EOL:     u8 = 255;

const MEDIALIB_HEADER_SIZE:   usize  = 1 + 2;
const IMAGELIB_HEADER_SIZE:   usize  = 1 + 2 + 2;
const FONTLIB_HEADER_SIZE:    usize  = 1 + 2 + 4;
const BORDERLIB_HEADER_SIZE:  usize  = 1 + 2 + 6 + 8;
const _PROGRAMLIB_HEADER_SIZE: usize  = 1 + 2;

#[derive(Copy, Clone)]
pub struct Library {    
    pub data: *const u8,
    pub len: usize
}

impl Library {
    pub fn clear(&mut self){
        self.data = ptr::null();
        self.len = 0;
    }
}

pub enum Identifier <'a> {
    Index(u8),
    Name(&'a str),
    Both(u8, &'a str,)
}

#[repr(u8)]
pub enum Ret {
    Ok = 0,
    BadHeader = 1,
    CRCFailure = 2,
    ShortHeaderRead = 3,
    ShortLibRead = 4,
    UnexpectedEnd = 5
}

pub struct Media {
    pub images: [Library; LIBRARY_MAX as usize],        
    pub fonts: [Library; LIBRARY_MAX as usize],    
    pub borders: [Library; LIBRARY_MAX as usize],  
    pub pages: [Library; LIBRARY_MAX as usize],
    #[cfg(feature = "vm")]
    pub programs: [Library; LIBRARY_MAX as usize],    
    pub drawing: media::image::Settings,
    pub typesetting: media::font::Settings    
}
impl Default for Media {
    fn default() -> Self { Media {
        images: [Library { data: ptr::null(), len: 0 }; LIBRARY_MAX as usize],        
        fonts: [Library { data: ptr::null(), len: 0 }; LIBRARY_MAX as usize],        
        borders: [Library { data: ptr::null(), len: 0 }; LIBRARY_MAX as usize],        
        pages: [Library { data: ptr::null(), len: 0 }; LIBRARY_MAX as usize],        
        #[cfg(feature = "vm")]
        programs: [Library { data: ptr::null(), len: 0 }; LIBRARY_MAX as usize],        
        drawing: media::image::Settings::default(),
        typesetting: media::font::Settings::default()        
    } }
}

impl Media {
    pub fn log(&self) {
        for i in 0 .. 255 {
            let image = self.get_image(Identifier::Index(i));
            if image.source != 255 && image.data.len() > 0 {
                log::info!("img #{}.{} [{:5}b] [{:5}b]", image.source, i, image.data.len(), image.mask.unwrap_or(&[]).len());                
            }
        }
        #[cfg(feature = "ninepatch")]
        for i in 0 .. 255 {
            let ninepatch = self.get_border(Identifier::Index(i));
            if ninepatch.source != 255 {
                let d = ninepatch.data;
                log::info!("brd #{}.{} [{:5}b]", ninepatch.source, i, d[0].len() + d[1].len() + d[2].len() + d[3].len() + d[4].len() + d[5].len() + d[6].len() + d[7].len());
            }
        }        
        for i in 0 .. 255 {
            let font = self.get_font(Identifier::Index(i));
            if font.source != 255 {                
                log::info!("fnt #{}.{} [{:5}b]", font.source, i, 0);
            }
        }                
        for i in 0 .. 255 {
            let page = self.get_page(i);
            if page.source != 255 {                
                log::info!("pag #{}.{} [{:5}b]", page.source, i, page.content.len());
            }
        }                
        #[cfg(feature = "vm")]
        for i in 0 .. 255 {
            let program = self.get_program(i);
            if program.source != 255 {                
                log::info!("prg #{}.{} [{:5}b]", program.source, i, program.data.len());
            }
        }                
    }
    
    pub fn clear(&mut self) {
        #[cfg(feature = "std")]
        self.clear_images();        
        #[cfg(all(feature = "std", feature = "ninepatch"))]
        self.clear_border();
        #[cfg(feature = "std")]
        self.clear_font();
        #[cfg(feature = "std")]
        self.clear_pages();
        #[cfg(all(feature = "std", feature = "vm"))]
        self.clear_programs();
        self.unload_pkg();
    }

    pub fn load_pkg(&mut self, pkg_ptr: *const u8, pkg_len: usize, source: u8) -> Ret {        
        let data = unsafe { slice::from_raw_parts(pkg_ptr.as_ref().unwrap(), pkg_len) };        
        let mut reader = Stream::new(data);

        let _magic = &data[0 .. 4];
        if reader.available() < 8 || str::from_utf8(&data[0 .. 4]).unwrap_or("").ne("PKG0") {
            log::error!("bad header");    
            return Ret::BadHeader;
        }        

        reader.set_pos(4);
        let length = reader.read_u32();
        let crc1 = reader.read_u16();
        let mut crc2 = 0x1002u16;
        for byte in &data[12..] {
            crc2 = crc2.wrapping_add(*byte as u16);
        }
        if crc1 != crc2 {
            log::info!("crc mismatch {:04x} != {:04x}", crc1, crc2);
            return Ret::CRCFailure;
        }        

        log::info!("loading media #{} [{}b]", source, length);

        if reader.ok == false {
            log::error!("short medialib header read");
            return Ret::ShortHeaderRead;
        }        

        loop {                
            if reader.available() >= MEDIALIB_HEADER_SIZE {
                let media_id = reader.read_u8();                                    
                if reader.ok == false {
                    log::error!("short media id read");
                    break;
                }
                if media_id == LIBMAGIC_EOL {                    
                    log::debug!("pkg end");
                    break;
                }
                let lib_length = reader.read_u16() as usize;                        
                if reader.ok == false {
                    log::error!("short lib length read");
                    return Ret::ShortLibRead;
                }
                if lib_length == 0 {                    
                    log::debug!("empty {:02x} lib", media_id);
                    break;
                }            
                
                let pos = reader.get_pos();
                let lib_ptr = data[pos .. ].as_ptr();
                let mut lib: &mut Library;
                if media_id == LIBMAGIC_IMAGE {                    
                    if source  >= LIBRARY_MAX {
                        log::warn!("too many img libraries");
                    }else{   
                        lib = &mut self.images[source as usize];                        
                        lib.data = lib_ptr;
                        lib.len = lib_length;                    
                        log::info!("found img lib #{} [{}b]", source, lib.len);                        
                    }
                }
                if media_id == LIBMAGIC_FONT {                     
                    if source >= LIBRARY_MAX {
                        log::warn!("too many fnt libraries");
                    }else{                                                   
                        lib = &mut self.fonts[source as usize];                        
                        lib.data = lib_ptr;
                        lib.len = lib_length;        
                        log::info!("found fnt lib #{} [{}b]", source, lib.len);                                    
                    }                    
                }     
                if media_id == LIBMAGIC_BORDER {                     
                    if source >= LIBRARY_MAX {
                        log::warn!("too many brd libraries");
                    }else{                                                   
                        lib = &mut self.borders[source as usize];                        
                        lib.data = lib_ptr;
                        lib.len = lib_length;        
                        log::info!("found brd lib #{} [{}b]", source, lib.len);                                    
                    }                    
                }
                if media_id == LIBMAGIC_PAGE {                    
                    if source  >= LIBRARY_MAX {
                        log::warn!("too many page libraries");
                    }else{   
                        lib = &mut self.pages[source as usize];                        
                        lib.data = lib_ptr;
                        lib.len = lib_length;                    
                        log::info!("found pag lib #{} [{}b]", source, lib.len);                        
                    }
                }
                #[cfg(feature = "vm")]
                if media_id == LIBMAGIC_PROGRAM {                    
                    if source  >= LIBRARY_MAX {
                        log::warn!("too many program libraries");
                    }else{   
                        lib = &mut self.programs[source as usize];                        
                        lib.data = lib_ptr;
                        lib.len = lib_length;                    
                        log::info!("found prg lib #{} [{}b]", source, lib.len);                        
                    }
                }
                reader.set_pos(pos + lib_length);
            }else{
                return Ret::UnexpectedEnd;
            }
        }
        return Ret::Ok;
    }

    pub fn unload_pkg(&mut self) {
        for i in 1 .. LIBRARY_MAX as usize {
            self.images[i].clear();
            self.borders[i].clear();
            self.fonts[i].clear();
            self.pages[i].clear();
            #[cfg(feature = "vm")]
            self.programs[i].clear();
        }
    }
}

pub mod image;
#[cfg(feature = "ninepatch")]
pub mod border;
pub mod font;
pub mod page;
#[cfg(feature = "vm")]
pub mod program;
#[cfg(feature = "inspect")]
pub mod inspect;

cfg_block! {
    if #[cfg(feature = "std")] {
        pub mod pkg;
        pub mod image_gen;
        pub mod font_gen;
        #[cfg(feature = "ninepatch")]
        pub mod border_gen;
        pub mod page_gen;        
        #[cfg(feature = "vm")]
        pub mod program_gen;        
    }else{}
}

