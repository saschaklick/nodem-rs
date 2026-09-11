use crate::media::*;

use virtmach::Program;

pub use u8 as ProgramIdx;

// pub struct Program<'a> {        
//     pub source: u8,
//     pub id: &'a str,
//     pub data: &'a [u8]    
// }
// impl Default for Program<'_> {
//     fn default() -> Self { Program { source: 255, id: "", data: b"" } }
// }

static EMPTY: [u8;0] = [0u8;0];

impl Media {
    pub fn get_program(&self, program_idx: ProgramIdx) -> Program<'_> {            
        let mut lib_idx = self.programs.len() as u8;
        for lib in self.programs.iter().rev() {
            lib_idx -= 1;
            if lib.data == ptr::null() {                
                continue;
            }                        
            let data = unsafe { slice::from_raw_parts(lib.data.as_ref().unwrap(), lib.len) };                        
            let mut reader = Stream::new(&data); 
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
                        if reader.available() >= 2 - 1 {                                         
                            let data_len= reader.read_u16() as usize;                                                                                    
                            if reader.available() >= data_len {                                
                                if idx == program_idx {                                                                            
                                    let pos = reader.get_pos();                                    
                                    return Program {
                                        source: lib_idx,                                                                        
                                        id,
                                        data: &data[pos .. pos + data_len],                                        
                                    }
                                }                            
                            }
                            reader.set_pos(reader.get_pos() + data_len);                
                        }else{
                            break;
                        }                    
                    }
                }else{
                    break;
                }
            }            
        }

        return Program {
            source: 255,
            id: "",
            data: &EMPTY
        };
    }
}