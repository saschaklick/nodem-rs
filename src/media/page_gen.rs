use bytes::{Buf, BufMut, BytesMut};

use crate::media::*;

static mut PAGE_IMPORT_LIB: Option<BytesMut> = None;

impl Media {
    pub(crate) fn insert_page(dest: &mut BytesMut, index: u8, id: &str, content: &[u8]) {
        dest.put_u8(index);
        dest.put_u8(id.len() as u8);
        dest.put(id.as_bytes());                
        dest.put_u16_ne(content.len() as u16);        
        dest.put(content);        
    }

    pub fn clear_pages(&mut self) {        
        unsafe {
            if PAGE_IMPORT_LIB.is_some() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                PAGE_IMPORT_LIB = Some(buf);
            }            
        }
    }

    pub fn remove_page(&mut self, index: u8) {
        self.import_page(index, "", b"");
    }

    pub fn import_page(&mut self, index: u8, name: &str, content: &[u8]) {        
        let lib: BytesMut;
        unsafe {
            if PAGE_IMPORT_LIB.is_none() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                PAGE_IMPORT_LIB = Some(buf);
            }
            lib = PAGE_IMPORT_LIB.take().unwrap();            
        }        

        let mut lib2 = BytesMut::new();
        let reader = &mut lib.clone();        
        let mut pos = 0;        
        loop {                        
            let idx = reader.get_u8();             
            if idx == 255 {
                Media::insert_page(&mut lib2, index, name, content);         
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
        
        self.pages[2].data = lib2.as_ptr();
        self.pages[2].len  = lib2.len();   

        unsafe {            
            PAGE_IMPORT_LIB = Some(lib2);
        }     
    }
}