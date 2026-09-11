use log;
use bytes::{BufMut, BytesMut, Buf};
use stream::Encoding;

extern crate alloc;
use alloc::vec::Vec;

use crate::media::*;
use crate::media::font::*;

static mut FONT_IMPORT_LIB: Option<BytesMut> = None;

const WHITE: [u8; 3] = [0xff, 0xff, 0xff];
const RED:   [u8; 3] = [0xff, 0x00, 0x00];
const BLACK: [u8; 3] = [0x00, 0x00, 0x00];
const BLUE:  [u8; 3] = [0x00, 0x00, 0xff];

#[derive(Clone)]
pub(crate) struct Glyph {
    id: u32,
    width: u8,
    height: u8,
    top: i8,
    left: i8,
    img: BytesMut
}

impl Media {
    pub(crate) fn insert_font(dest: &mut BytesMut, index: u8, id: &str, mono_width: u8, base_height: u8, full_height: u8, glyphs: Vec<Glyph>) {
        let mut lib = BytesMut::new();
        let mut pack_cnt = 0u8;

        if glyphs.len() > 0 {
            let mut last_id = glyphs[0].id - 1;
            let mut pack_id = glyphs[0].id;
            let mut pack_len = 0usize;                    
            let mut pack_size = 0usize;
            let mut indexes = BytesMut::new();
            let mut data = BytesMut::new();
            for glyph in glyphs {                
                if glyph.id != last_id + 1 {                
                    lib.put_u8(pack_id as u8);
                    lib.put_u8((pack_id + pack_len as u32 - 1) as u8);
                    lib.put_u16_ne((indexes.len() + data.len()) as u16);
                    lib.put(indexes.clone());
                    lib.put(data.clone());
                    pack_id = glyph.id;
                    pack_len = 0;
                    pack_size = 0;
                    pack_cnt += 1;
                    indexes.clear();
                    data.clear();
                }
                //log::info!("{}", glyph.id);

                pack_size += GLYPH_HEADER_SIZE + glyph.img.len();
                indexes.put_u16_ne(pack_size as u16);
                data.put_u8(glyph.width);
                data.put_u8(glyph.height);            
                data.put_i8(glyph.left);
                data.put_i8(glyph.top);
                data.put(glyph.img);
                pack_len += 1;            

                last_id = glyph.id;            
            }
            if pack_len > 0 {            
                lib.put_u8(pack_id as u8);
                lib.put_u8((pack_id + pack_len as u32 - 1) as u8);
                lib.put_u16_ne((indexes.len() + data.len()) as u16);
                lib.put(indexes.clone());
                lib.put(data.clone());
                pack_cnt += 1;
            }   
        }           
        log::info!("imported font {} (\"{}\") with {} packs [{}b]", index, id, pack_cnt, FONT_HEADER_SIZE + lib.len() + id.len());
        dest.put_u8(index);
        dest.put_u8(id.len() as u8);            
        dest.put(id.as_bytes());            
        dest.put_u16_ne((FONT_HEADER_SIZE + lib.len()) as u16);
        dest.put_u8(pack_cnt);
        dest.put_u8(mono_width);
        dest.put_u8(base_height);
        dest.put_u8(full_height);
        dest.put(lib);                
    }

    pub fn clear_font(&mut self){
        unsafe {
            if FONT_IMPORT_LIB.is_some() {            
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                FONT_IMPORT_LIB = Some(buf);
            }
        }
    }

    pub fn remove_font(&mut self, index:u8) {
        self.import_font(index, "", b"", 0, 0);   
    }

    pub fn import_font(self: &mut Media, index: u8, name: &str, pixels: &[u8], width: u32, height: u32) {
        let lib: BytesMut;
        unsafe {
            if FONT_IMPORT_LIB.is_none() {
                let mut buf = BytesMut::zeroed(0);
                buf.put_u8(255);
                FONT_IMPORT_LIB = Some(buf);
            }
            lib = FONT_IMPORT_LIB.take().unwrap();            
        }

        let mut cols_x: Vec<usize> = Vec::new();
        let mut cols_l: Vec<usize> = Vec::new();
        cols_l.push(0);
        for x in 0 .. width as usize {
            let pixel = &pixels[x * 4 .. x * 4 + 3];            
            if pixel == RED {
                if *cols_l.last().unwrap() == 0 { cols_x.push(x); }                
                *cols_l.last_mut().unwrap() += 1;
            }else
            if pixel == BLACK { if cols_l[&cols_l.len() - 1] > 0 { cols_l.push(0); } }
        }
        if *cols_l.last().unwrap() == 0 { cols_l.pop(); }

        let mut ids_y: Vec<usize> = Vec::new();
        let mut ids_l: Vec<usize> = Vec::new();
        let mut rows_y: Vec<usize> = Vec::new();
        let mut rows_l: Vec<usize> = Vec::new();
        ids_l.push(0);
        rows_l.push(0);
        for y in 0 .. height as usize {
            let pixel = &pixels[y * width as usize * 4 .. y * width as usize * 4 + 3];            
            if pixel == BLUE {
                if *ids_l.last().unwrap() == 0 { ids_y.push(y); }                
                *ids_l.last_mut().unwrap() += 1;
            }else
            if pixel == RED {
                if *rows_l.last().unwrap() == 0 { rows_y.push(y); }                
                *rows_l.last_mut().unwrap() += 1;
            }else        
            if pixel == BLACK {
                if ids_l[&ids_l.len() - 1] > 0 { ids_l.push(0); }
                if rows_l[&rows_l.len() - 1] > 0 { rows_l.push(0); }
            }
        }
        if *ids_l.last().unwrap() == 0 { ids_l.pop(); }
        if *rows_l.last().unwrap() == 0 { rows_l.pop(); }
    
        log::debug!("cols {:?} {:?}", cols_x, cols_l);
        log::debug!("ids  {:?} {:?}", ids_y, ids_l);
        log::debug!("rows {:?} {:?}", rows_y, rows_l);

        let mut glyphs: Vec<Glyph> = Vec::new();        
        let mut min_x = usize::MAX;
        let mut max_x = usize::MIN;
        let mut min_y = usize::MAX;
        let mut max_y = usize::MIN;

        for row in 0 .. ids_y.len() {
            for col in 0 .. cols_x.len() {  
                let mut g_id: u32 = 0;
                let mut mask: u32 = 0b1;
                for y in ids_y[row] .. ids_y[row] + ids_l[row] {
                    for x in cols_x[col] .. cols_x[col] + cols_l[col] {
                        let pixel = &pixels[(y * width as usize + x) * 4 .. (y * width as usize + x) * 4 + 3];                                    
                        if pixel == BLUE {
                            g_id |= mask;
                        }
                        mask <<= 1;                        
                    }       
                }
                if g_id > 0 {
                    let x_o = cols_x[col];
                    let y_o = rows_y[row];
                    
                    let mut x_0 = cols_x[col];                    
                    let mut x_1 = cols_x[col] + cols_l[col];
                    let mut not_empty = false;
                    loop {                                                
                        for y in rows_y[row] .. rows_y[row] + rows_l[row] {
                            let pixel = &pixels[(y * width as usize + x_0) * 4 .. (y * width as usize + x_0) * 4 + 3];
                            if pixel == WHITE { not_empty = true; }
                        }
                        if not_empty == true || x_0 == x_1 { break; }
                        x_0 += 1;
                    }                    
                    not_empty = false;
                    loop {                                                                        
                        for y in rows_y[row] .. rows_y[row] + rows_l[row] {
                            let pixel = &pixels[(y * width as usize + x_1) * 4 .. (y * width as usize + x_1) * 4 + 3];
                            if pixel == WHITE { not_empty = true; }
                        }
                        if not_empty == true || x_0 == x_1 { break; }
                        x_1 -= 1;
                    }
                    let mut y_0 = rows_y[row];
                    let mut y_1 = rows_y[row] + rows_l[row];
                    not_empty = false;
                    loop {                                                
                        for x in x_0 .. x_1 + 1 {
                            let pixel = &pixels[(y_0 * width as usize + x) * 4 .. (y_0 * width as usize + x) * 4 + 3];
                            if pixel == WHITE { not_empty = true; }
                        }
                        if not_empty == true || y_0 == y_1 { break; }
                        y_0 += 1;
                    }                    
                    not_empty = false;
                    loop {                                                                                              
                        for x in x_0 .. x_1 + 1 {
                            let p_i = (y_1 * width as usize + x) * 4;
                            if p_i + 3 >= pixels.len() { break; }
                            let pixel = &pixels[p_i .. p_i + 3];
                            if pixel == WHITE { not_empty = true; }
                        }
                        if not_empty == true || y_0 == y_1 { break; }
                        y_1 -= 1;
                    }

                    let g_w = x_1 - x_0;
                    let g_h = y_1 - y_0;                    
                    
                    if x_0 - x_o == cols_l[col] {
                        log::warn!("skipped empty glyph {}", g_id);
                    }else{
                        if (x_0 - x_o) < min_x { min_x = x_0 - x_o; }
                        if (x_1 - x_o) > max_x { max_x = x_1 - x_o; }
                        if (y_0 - y_o) < min_y { min_y = y_0 - y_o; }
                        if (y_1 - y_o) > max_y { max_y = y_1 - y_o; }                        

                        let mut img = BytesMut::new();
                        let mut byte = 0u8;
                        let mut mask = 0b10000000u8;
                        img.put_u8(Encoding::Raw as u8);
                        for y in y_0 .. y_1 + 1 {
                            for x in x_0 .. x_1 + 1 {
                                let pixel = &pixels[(y * width as usize + x) * 4 .. (y * width as usize + x) * 4 + 3];
                                if pixel == WHITE {
                                    byte |= mask;
                                }
                                if mask == 0b00000001 {
                                    img.put_u8(byte);
                                    byte = 0;
                                    mask = 0b10000000;
                                } else { mask >>= 1; }
                            }
                        }
                        if mask != 0b10000000 { img.put_u8(byte); }

                        let glyph = Glyph {
                            id: g_id,
                            width: g_w as u8 + 1,
                            height: g_h as u8 + 1,
                            top: (y_0 - y_o) as i8,
                            left: (x_0 - x_o) as i8,
                            img: img
                        };
                        
                        log::debug!("glyph {:5} '{}': {}x{} [{}b] at ({},{}),({},{})", char::from_u32(glyph.id).unwrap(), glyph.id, glyph.width, glyph.height, glyph.img.len(), x_0, y_0, x_1, y_1);                        

                        glyphs.push(glyph);
                    }
                }
            }
        }        

        let mut lib2 = BytesMut::new();
        let reader = &mut lib.clone();        
        let mut pos = 0;        
        loop {                        
            let idx = reader.get_u8();              
            if idx == 255 {
                if glyphs.len() == 0 {
                    if width > 0 || height > 0 {
                        log::warn!("skipping font {}: no glyphs found", index);
                    }else{                        
                        Media::insert_font(&mut lib2, index, "", 0, 0, 0, [].to_vec());                                                
                    }
                }else{                    
                    glyphs.sort_by(|a, b| a.id.cmp(&b.id));                        
                    
                    let mut base_height: Option<u8> = None;
                    for glyph in &mut glyphs {
                        glyph.top -= min_y as i8;                
                        glyph.left -= min_x as i8;                
                        let g_char = char::from_u32(glyph.id).unwrap();
                        match g_char{
                            'A' | 'W' | '0' | '!' => { base_height = Some(glyph.height); }
                            _ => {}
                        }                
                        log::trace!("{} '{}' h: {:2} w: {:2} t: {:2} l: {:2}", glyph.id, g_char, glyph.width, glyph.height, glyph.top, glyph.left);
                    }
                    if base_height.is_none() { base_height = Some(glyphs[0].height); }  
                    let mono_width = max_x - min_x + 1;         
                    let full_height = max_y - min_y + 1;                    
                                        
                    Media::insert_font(&mut lib2, index, name, mono_width as u8, base_height.unwrap(), full_height as u8, glyphs);                                     
                    
                    log::debug!("     #{}.{} m_w: {} b_h: {} f_h: {}", 2, index, mono_width, base_height.unwrap(), full_height);                                        
                }                
                lib2.put_u8(255);
                break;
            }            

            let id_len = reader.get_u8() as usize;            
            reader.advance(id_len);
            let content_len = reader.get_u16_ne() as usize;            
            reader.advance(content_len);            
            let len = 1 + 1 + id_len as usize + 2 + content_len as usize;
            if idx != index {
                lib2.put(&lib[pos..pos + len]);               
            }
            pos += len;    
        }                      

        self.fonts[2].data = lib2.as_ptr();
        self.fonts[2].len = lib2.len();                       

        unsafe {            
            FONT_IMPORT_LIB = Some(lib2);
        }     
    }

}