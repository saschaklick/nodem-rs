use bytes::{Buf, BufMut, BytesMut};

use crate::media::*;

static mut IMAGE_IMPORT_LIB: Option<BytesMut> = None;

impl Media {
    pub(crate) fn insert_image(dest: &mut BytesMut, index: u8, id: &str, width: u8, height: u8, data: &[u8], mask: &[u8] ) {        
        dest.put_u8(index);
        dest.put_u8(id.len() as u8);
        dest.put(id.as_bytes());
        dest.put_u16_ne(data.len() as u16 + 3);
        dest.put_u16_ne(mask.len() as u16);
        dest.put_u8(1);
        dest.put_u8(width);
        dest.put_u8(height);
        dest.put(data);
        dest.put(mask);                
    }

    pub fn clear_images(&mut self) {        
        unsafe {        
            if IMAGE_IMPORT_LIB.is_some() {                
                let mut lib = BytesMut::new();                
                lib.put_u8(255);
                IMAGE_IMPORT_LIB = Some(lib);
            }
        }
    }

    pub fn remove_image(&mut self, index: u8) {
        self.import_image(index, "", b"", 0, 0, false, 0, 0);
    }

    pub fn import_image(&mut self, index: u8, name: &str, pixels: &[u8], width: u8, height: u8, alpha: bool, threshold_0: u8, threshold_1: u8) {
        let lib: BytesMut;
        unsafe {
            if IMAGE_IMPORT_LIB.is_none() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);                
                IMAGE_IMPORT_LIB = Some(buf);
            }
            lib = IMAGE_IMPORT_LIB.take().unwrap();            
        }        

        let mut data = BytesMut::new();
        let mut mask = BytesMut::new();

        data.put_u8(Encoding::Raw as u8);
        if alpha {
            mask.put_u8(Encoding::Raw as u8);
        }        
        
        let mut data_byte: u8 = 0;        
        let mut mask_byte: u8 = 0;        
        let mut pos: usize = 0;
        let mut has_alpha = false;
        let pixel_size = if alpha { 4 } else { 3 };
        let pixel_cnt = width as usize * height as usize;
        let thresholds= [threshold_0 as i16 * 3, threshold_1 as i16 * 3];            
        for i in 0 .. pixel_cnt {
            if pixels.len() < pos + pixel_size {
                break;
            }            
            let r = pixels[pos + 0];
            let g = pixels[pos + 1];
            let b = pixels[pos + 2];            
            if (r as i16 + g as i16 + b as i16) > thresholds[((i % width as usize) + ((i / width as usize) % 2)) % 2] {
                data_byte |= 1 << (7 - (i % 8));
            }
            if alpha {
                let a = pixels[pos + 3];
                if a != 255 {
                    has_alpha = true;
                }
                if a > 0 {
                    mask_byte |= 1 << (7 - (i % 8));
                }
            }
            if i % 8 == 7 || i == pixel_cnt - 1 {
                data.put_u8(data_byte);
                data_byte = 0;
                if alpha {
                    mask.put_u8(mask_byte);
                    mask_byte = 0;
                }
            }
            pos += pixel_size;
        }  

        if !has_alpha {
            mask.clear();
        }                
        
        let mut lib2 = BytesMut::new();
        let reader = &mut lib.clone();
        let mut pos = 0;        
        loop {                        
            let idx = reader.get_u8();                                        
            if idx == 255 {
                Media::insert_image(&mut lib2, index, name, width, height, &data, &mask);   
                lib2.put_u8(255);
                break;
            }            
            let id_len = reader.get_u8() as usize;                        
            reader.advance(id_len);
            let data_len = reader.get_u16_ne() as usize;            
            let mask_len = reader.get_u16_ne() as usize;                        
            reader.advance(data_len + mask_len);            
            let len = 1 + 1 + id_len as usize + 2 + 2 + data_len as usize + mask_len as usize;
            if idx != index {
                lib2.put(&lib[pos..pos + len]);               
            }
            pos += len;                                           
        }                
            
        self.images[2].data = lib2.as_ptr();
        self.images[2].len = lib2.len();   

        unsafe {            
            IMAGE_IMPORT_LIB = Some(lib2);
        }     
    }    
}