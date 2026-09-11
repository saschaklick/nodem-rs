use bytes::{Buf, BufMut, BytesMut};

use crate::media::*;

static mut PROGRAM_IMPORT_LIB: Option<BytesMut> = None;

impl Media {
    pub(crate) fn insert_program(dest: &mut BytesMut, index: u8, id: &str, data: &[u8]) {
        dest.put_u8(index);
        dest.put_u8(id.len() as u8);
        dest.put(id.as_bytes());        
        dest.put_u16_ne(data.len() as u16);        
        dest.put(data);        
    }

    pub fn clear_programs(&mut self) {
        unsafe {
            if PROGRAM_IMPORT_LIB.is_some() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                PROGRAM_IMPORT_LIB = Some(buf);
            }            
        }
    }

    pub fn remove_program(&mut self, index: u8) {
        self.import_program(index, "", b"");        
    }

    pub fn import_program(&mut self, index: u8, name: &str, data: &[u8]) {
        let lib: BytesMut;
        unsafe {
            if PROGRAM_IMPORT_LIB.is_none() {
                let mut buf = BytesMut::new();
                buf.put_u8(255);
                PROGRAM_IMPORT_LIB = Some(buf);
            }
            lib = PROGRAM_IMPORT_LIB.take().unwrap();            
        }        

        let mut lib2 = BytesMut::new();
        let reader = &mut lib.clone();        
        let mut pos = 0;        
        loop {                        
            let idx = reader.get_u8();             
            if idx == 255 {
                Media::insert_program(&mut lib2, index, name, data);         
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
        
        self.programs[2].data = lib2.as_ptr();
        self.programs[2].len  = lib2.len();   

        unsafe {            
            PROGRAM_IMPORT_LIB = Some(lib2);
        }             
    }
}