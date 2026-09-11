use crate::media::*;

impl Media {
    pub fn get_border(&self, ident: Identifier) -> NinePatch <'_> {
        let mut lib_idx = self.borders.len() as u8;
        for lib in self.borders.iter().rev() {
            lib_idx -= 1;
            if lib.data == ptr::null() {                
                continue;
            }            
            let data = unsafe { slice::from_raw_parts(lib.data.as_ref().unwrap(), lib.len) };
            let mut reader = Stream::new(data);

            loop {                                
                if reader.available() >= 2 {                                        
                    let idx = reader.read_u8();                                                                            
                    let id_len = reader.read_u8() as usize;                    
                    if reader.available() >= id_len {                        
                        let id = str::from_utf8(&data[reader.get_pos() .. reader.get_pos() + id_len]).unwrap_or("");                        
                        reader.set_pos(reader.get_pos() + id_len);
                        if reader.ok == false || idx == 255 {
                            break;
                        }                                             
                        
                        if reader.available() >= BORDERLIB_HEADER_SIZE - 1 {                                                                                                                                               
                            let border_len= reader.read_u16() as usize;                          
                            let border_pos = reader.get_pos();                                                                           
                            if reader.available() >= border_len {                                                                                                                            
                                
                                if match ident {
                                    Identifier::Index(index) => index == idx,
                                    Identifier::Name(name) => !name.is_empty() && name == id,
                                    Identifier::Both(index, name) => index == idx || (!name.is_empty() && name == id)
                                } {                                    
                                    let values = [0u8;6].map(|_| { reader.read_u8() });                                                                                                                                                    
                                    return if values[0] == 0 || values[1] == 0 || values[2] == 0 || values[3] == 0 {
                                        NinePatch::default()
                                    } else {
                                        let mut ninepatch= NinePatch {
                                            source: lib_idx,
                                            id : id,
                                            n_h: values[0],
                                            e_w: values[1],
                                            s_h: values[2],
                                            w_w: values[3],
                                            x_w: values[4],
                                            y_h: values[5],
                                            data: [&[0]; 8]                                    
                                        };
                                        let mut pos: usize = reader.get_pos() + 8;
                                        for i in 0 .. 8 {                                                              
                                            let data_len = reader.read_u8() as usize;                                        
                                            ninepatch.data[i] = &data[pos..pos + data_len];
                                            pos += data_len;
                                        }                                
                                        ninepatch
                                    };
                                }                                       
                            }else{
                                break;
                            }                                                                    
                            reader.set_pos(border_pos + border_len);                        
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

        return NinePatch::default();
    }
}