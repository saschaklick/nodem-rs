use log;
use bytes::{BufMut, Bytes, BytesMut, Buf};

use crate::media::*;

static mut BORDER_IMPORT_LIB: Option<BytesMut> = None;

const RED:    [u8; 3] = [0xff, 0x00, 0x00];    
const _BLACK: [u8; 3] = [0x00, 0x00, 0x00];        
const _BLUE:  [u8; 3] = [0x00, 0x00, 0xff];   

impl Media {
    pub(crate) fn insert_border(dest: &mut BytesMut, index: u8, id: &str, n_h: u8, e_w: u8, s_h: u8, w_w: u8, x_w: u8, y_h: u8, data: &[Bytes; 8]) {
        dest.put_u8(index);
        dest.put_u8(id.len() as u8);
        dest.put(id.as_bytes());
        let mut len: usize = 6 + 8;
        for i in 0 .. 8 {
            len += data[i].len();
        }
        dest.put_u16_ne(len as u16);
        dest.put_u8(n_h);
        dest.put_u8(e_w);
        dest.put_u8(s_h);
        dest.put_u8(w_w);
        dest.put_u8(x_w);
        dest.put_u8(y_h);
        for i in 0 .. 8 {
            dest.put_u8(data[i].len() as u8);
        }
        for i in 0 .. 8 {
            dest.put(data[i].slice(0..));
        }               
    }

    pub fn clear_border(&mut self) {
        unsafe {
            if BORDER_IMPORT_LIB.is_some() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                BORDER_IMPORT_LIB = Some(buf);
            }            
        }
    }

    pub fn remove_border(&mut self, index:u8) {
        self.import_border(index, "", b"", 0, 0);    
    }

    pub fn import_border(self: &mut Media, index: u8, id: &str, pixels: &[u8], width: u8, height: u8) {
        let lib: BytesMut;
        unsafe {
            if BORDER_IMPORT_LIB.is_none() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                BORDER_IMPORT_LIB = Some(buf);
            }
            lib = BORDER_IMPORT_LIB.take().unwrap();            
        }

        let mut n_h: u8 = 0;
        let mut e_w: u8 = 0;
        let mut s_h: u8 = 0;
        let mut w_w: u8 = 0;     
        
        let mut w_s: u8 = 0;
        let mut e_s: u8 = 0;
        let mut progress = 0 as usize;        
        for x in 0 .. width as usize {
            let pixel = &pixels[x * 4..x * 4 + 3];
            match progress {
                0 => if pixel == RED { w_w = 1; w_s = x as u8; progress = 1; }
                1 => if pixel == RED { w_w += 1; } else { progress = 2; }
                2 => if pixel == RED { e_w = 1; e_s = x as u8; progress = 3; }
                3 => if pixel == RED { e_w += 1; } else { progress = 4; }
                _ => { break; }
            }            
        }             
        let mut n_s: u8 = 0;
        let mut s_s: u8 = 0;
        let mut progress = 0 as usize;        
        for y in 0 .. height as usize {
            let pixel = &pixels[y * width as usize * 4..y * width as usize * 4 + 3];
            match progress {
                0 => if pixel == RED { n_h = 1; n_s = y as u8; progress = 1; }
                1 => if pixel == RED { n_h += 1; } else { progress = 2; }
                2 => if pixel == RED { s_h = 1; s_s = y as u8; progress = 3; }
                3 => if pixel == RED { s_h += 1; } else { progress = 4; }
                _ => { break; }
            }            
        }     

        let x_w: u8 = e_s - w_s - w_w;
        let y_h: u8 = s_s - n_s - n_h;

        const EMPTY: Bytes = Bytes::new();
        
        let mut data: [Bytes;8] = [EMPTY; 8];
        for i in 0 .. 8 {
            let mut buf = BytesMut::new();
            buf.put_u8(Encoding::Raw as u8);
            let mut byte = 0;
            let mut bit = 0b10000000;
            match i {
                0 => {
                    for x in (w_s + w_w) as usize .. e_s as usize { for y in n_s as usize .. (n_s + n_h) as usize { 
                        let p = (y * width as usize + x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                1 => {
                    for y in n_s as usize .. (n_s + n_h) as usize { for x in 0 .. e_w as usize {
                        let p = (y * width as usize + (e_s + e_w - 1) as usize - x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                2 => {
                    for y in (n_s + n_h) as usize .. s_s as usize { for x in 0 .. e_w as usize {
                        let p = (y * width as usize + ((e_s + e_w - 1) as usize - x)) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                3 => {
                    for y in 0 .. s_h as usize { for x in 0 .. e_w as usize {
                        let p = (((s_s + s_h - 1) as usize - y) * width as usize + ((e_s + e_w - 1) as usize - x)) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                4 => {
                    for x in (w_s + w_w) as usize .. e_s as usize { for y in 0 .. s_h as usize { 
                        let p = (((s_s + s_h - 1) as usize - y) * width as usize + x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                5 => {
                    for y in 0 as usize .. s_h as usize { for x in w_s as usize .. (w_s + w_w) as usize {
                        let p = (((s_s + s_h - 1) as usize - y) * width as usize + x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                6 => {
                    for y in (n_s + n_h) as usize .. s_s as usize { for x in w_s as usize .. (w_s + w_w) as usize {
                        let p = (y * width as usize + x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                7 => {
                    for y in n_s as usize .. (n_s + n_h) as usize { for x in w_s as usize .. (w_s + w_w) as usize {
                        let p = (y * width as usize + x) * 4; if pixels[p..p + 3] != [0x00, 0x00, 0x00] { byte |= bit; }
                        bit >>= 1;
                        if bit == 0 { buf.put_u8(byte); bit = 0b10000000; byte = 0; }
                    } }
                    if bit != 0b10000000 { buf.put_u8(byte); }
                }
                _ => {}
            }

            
            data[i] = buf.into();
        }
           
        let mut lib2 = BytesMut::new();
        let reader = &mut lib.clone();        
        let mut pos = 0;        
        loop {                        
            let idx = reader.get_u8();             
            if idx == 255 {
                Media::insert_border(&mut lib2, index, id, n_h, e_w, s_h, w_w, x_w, y_h, &data);         
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
        
        self.borders[2].data = lib2.as_ptr();
        self.borders[2].len  = lib2.len();   

        log::debug!("border #{}.{} n: {} e: {} s: {}, w: {}, x: {}, y: {} n: {}b ne: {}b", 2, index, n_h, e_w, s_h, w_w, x_w, y_h, data[0].len(), data[1].len());

        unsafe {            
            BORDER_IMPORT_LIB = Some(lib2);
        }     
    }
}