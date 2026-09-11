use log;
//use bytes::{BytesMut};

use crate::media::*;

pub use u8 as FontBaseH;
pub use u8 as FontFullH;
pub use u8 as FontMonoW;
pub use u8 as GlyphIdx;
pub use u8 as GlyphW;
pub use u8 as GlyphH;
pub use u8 as GlyphL;

pub const FONT_HEADER_SIZE: usize = 4;
pub const GLYPH_HEADER_SIZE: usize = 4;

static DUMMY_FONT: [u8; 1] = [ 255 ];
pub static DUMMY_GLYPH: [u8; 2] = [ 0, 0 ];
pub struct Font <'a> {
    pub source: u8,
    pub id: &'a str,
    pub data: &'a [u8],    
    pub pack_count: u8,
    pub base_height: u8,
    pub full_height: u8,
    pub mono_width: u8  
}
impl Default for Font <'_> {
    fn default() -> Self { Font { source: 255, id: "", data: &DUMMY_FONT, pack_count: 1, base_height: 4, full_height: 4, mono_width: 3 } }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct Glyph <'a> {
    id: u32,
    width: u8,
    height: u8,
    top: i8,
    left: i8,
    img: &'a[u8]
}

#[derive(Copy, Clone)]
pub enum Alignment {
    Proportional,
    Start,
    Center,
    End
}

#[derive(Copy, Clone)]
pub struct Settings {    
    pub default_idx: FontIdx,
    pub active_idx: FontIdx,
    pub mono_spaced: Alignment,
    pub fixed_height: bool,
    pub glyph_spacing: u8,
    pub line_spacing: u8,
    pub space_width: u8,    
    pub placeholders: bool,
    pub inverted: bool  
}
impl Default for Settings {
    fn default() -> Self { Settings { default_idx: 0, active_idx: 0, mono_spaced: Alignment::Proportional, fixed_height: false, glyph_spacing: 1, line_spacing: 1, space_width: 3, placeholders: true, inverted: false } }
}

impl Media {
    pub fn get_font(&self, ident: Identifier) -> Font <'_> {
        let mut lib_idx = self.fonts.len() as u8;
        for lib in self.fonts.iter().rev() {
            lib_idx -= 1;
            if lib.data == ptr::null() {                
                continue;
            }
            let data = unsafe { slice::from_raw_parts(lib.data.as_ref().unwrap(), lib.len) };            
            let mut reader = Stream::new(data);

            loop {                                
                if reader.available() >= 1 {                                        
                    let idx = reader.read_u8();                           
                    if idx == 255 {
                        break;
                    }   
                    let id_len = reader.read_u8() as usize;
                    if reader.available() >= id_len {                    
                        let id = str::from_utf8(&data[reader.get_pos() .. reader.get_pos() + id_len]).unwrap_or("");
                        reader.set_pos(reader.get_pos() + id_len);
                        if reader.available() >= FONTLIB_HEADER_SIZE - 1 + 1 + id.len() {                                                                                                      
                            let font_length= reader.read_u16() as usize;
                            let font_pos = reader.get_pos();                               
                            if reader.available() > font_length {                                                                             
                                if match ident {
                                    Identifier::Index(index) => index == idx,
                                    Identifier::Name(name) => !name.is_empty() && name == id,
                                    Identifier::Both(index, name) => index == idx || (!name.is_empty() && name == id)
                                } {                                                                                          
                                    return if font_length < 4 {
                                        Font::default()
                                    } else {
                                        Font {
                                            source: lib_idx,
                                            id : id,
                                            data: &data[font_pos + 4 .. font_pos + font_length],
                                            pack_count: reader.read_u8(),
                                            mono_width: reader.read_u8(),
                                            base_height: reader.read_u8(),
                                            full_height: reader.read_u8()
                                        }
                                    };                                    
                                }                                       
                            }else{
                                break;
                            }                                                                    
                            reader.set_pos(font_pos + font_length);
                        }else{
                            break;
                        }
                    }else{
                        break;
                    }                    
                }else{
                    break;
                }                
            }         
        }

        return Font::default();
    }

    pub fn get_glyph <'a> (&self, font: &'a Font, char: char) -> &'a [u8] { 
        let data = &font.data;
        let mut reader = Stream::new(data);        
        let mut first_no: u8;
        let mut last_no: u8;

        for _p_i in 0 .. font.pack_count {            
            first_no = reader.read_u8();
            last_no = reader.read_u8();  
            
            if first_no > last_no {
                break;
            }          
            let pack_len = reader.read_u16();
            let pack_pos = reader.get_pos();            
            reader.set_pos(pack_pos + (last_no - first_no) as usize * 2);
            let pack_end = reader.read_u16();                        
            if char as u8 >= first_no && char as u8 <= last_no {            
                let glyph_idx = char as u8 - first_no;
                let data_pos = pack_pos + ((last_no - first_no + 1) as usize * 2);
                let mut glyph_pos: u16 = 0;
                if glyph_idx > 0 {                    
                    reader.set_pos(pack_pos + (glyph_idx - 1) as usize * 2);
                    glyph_pos = reader.read_u16();                                   
                }
                reader.set_pos(pack_pos + glyph_idx as usize * 2);
                let glyph_pos_next = reader.read_u16();                
                
                if glyph_pos >= glyph_pos_next {
                    log::error!("glyph index malformed {} {} {}", char, glyph_pos, glyph_pos_next);
                    break;
                }else
                if (glyph_pos + GLYPH_HEADER_SIZE as u16) >= pack_end {                    
                    log::error!("glyph overflow {} {} {} {:02X?}", char, glyph_pos, pack_end, &data[0 .. 10]);
                    break;
                }else{
                    let start = data_pos + glyph_pos as usize;
                    let end = start + (glyph_pos_next - glyph_pos) as usize;
                    if data.len() >= data_pos + glyph_pos as usize + (glyph_pos_next - glyph_pos) as usize {                        
                        let glyph = &data[start .. end];
                        //log::debug!("{} {} {} - {} {}", char as u8, first_no, last_no, glyph_pos, glyph_pos_next);                                      
                        return glyph;                    
                    }else{
                        log::error!("glyph data too short ({}b), expected ({}b)", end, data.len());                    
                        return &DUMMY_GLYPH;
                    }
                }       
            }
            reader.set_pos(pack_pos + pack_len as usize);
        }        
        return &DUMMY_GLYPH;
    }
}
