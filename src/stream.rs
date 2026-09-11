#[repr(u8)]
#[derive(PartialEq)]
pub enum Encoding {
    Raw  = 0x00,
    RLEFixed = 0x01,
    EOL
}

pub struct Stream<'a> {
    data: &'a [u8],
    pub ok: bool,
    pos: usize    
}
impl <'a> Stream<'a> {
    pub fn new(data: &'a [u8]) -> Self {   
        return Self { data: data, ok: true, pos: 0 };
    }

    pub fn get_pos(&self)  -> usize {
        return self.pos;
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
        self.ok = self.pos < self.data.len();
    }

    pub fn available(&self) -> usize {
        return if self.pos >= self.data.len() { 0 } else { self.data.len() - self.pos }
    }

    pub fn read_u8(&mut self) -> u8 {        
        if self.pos < self.data.len() {
            self.ok = true;            
            let byte = self.data[self.pos];
            self.pos += 1;
            return byte;
        } else {
            self.ok = false;            
            return 0;
        }
    }

    pub fn read_i8(&mut self) -> i8 {        
        if self.pos < self.data.len() {
            self.ok = true;            
            let byte = self.data[self.pos];
            self.pos += 1;
            return if byte < 127 { byte as i8} else { -((256 - byte as i16)) as i8 };
        } else {
            self.ok = false;            
            return 0;
        }
    }

    pub fn read_u16(&mut self) -> u16 {        
        if self.pos + 2 < self.data.len() {
            self.ok = true;
            let word = (self.data[self.pos] as u16) << 0 | (self.data[self.pos + 1] as u16) << 8;
            self.pos += 2;            
            return word;
        } else {
            self.ok = false;
            return 0;
        }
    }

    pub fn read_u32(&mut self) -> u32 {        
        if self.pos + 4 < self.data.len() {
            self.ok = true;
            let long = (self.data[self.pos] as u32) << 0 | (self.data[self.pos + 1] as u32) << 8 | (self.data[self.pos + 2] as u32) << 16 | (self.data[self.pos + 3] as u32) << 24;
            self.pos += 4;            
            return long;
        } else {
            self.ok = false;
            return 0;
        }
    }
}

pub struct BinaryStream<'a> {
    data: &'a [u8],
    pub ok: bool,
    pos: usize,        
    rpt_cnt: usize,
    rpt_val: u8,
    raw_cnt: usize,
    encoding: Encoding
}
impl <'a> BinaryStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {   
        let mut reader = Self { data: data, ok: false, pos: 0, rpt_cnt: 0, rpt_val: 0, raw_cnt: 0, encoding: Encoding::EOL };
        
        reader.reset();
    
        return reader;
    }    
    
    pub fn read_bit(&mut self) -> u8 {
        if self.pos / 8 >= self.data.len() {
            self.ok = false;
            return 0;
        }
        match self.encoding {
            Encoding::Raw => {
                let res = (self.data[self.pos / 8] >> (7 - (self.pos % 8))) & 1;
                self.pos += 1;
                res
            },
            Encoding::RLEFixed => {
                if self.rpt_cnt == 0 && self.raw_cnt == 0 {
                    if (self.data[self.pos / 8] >> 7) & 1 == 0 {
                        self.raw_cnt = 7;                                            
                    } else {
                        self.rpt_val = (self.data[self.pos / 8] >> 6) & 1;
                        self.rpt_cnt = (self.data[self.pos / 8] & 0b00111111) as usize;                                               
                    }
                }
                if self.rpt_cnt > 0 {
                    self.rpt_cnt -= 1;
                    if self.rpt_cnt == 0 { self.pos += 8; }
                    self.rpt_val
                } else {                    
                    let ret = self.data[self.pos / 8] >> (self.pos % 8) & 1;
                    self.pos += 1;
                    self.raw_cnt -= 1;
                    if self.raw_cnt == 0 { self.pos += 1; }
                    ret
                }            
            },
            _ => { 0 }
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.rpt_cnt = 0;
        self.raw_cnt = 0;
        
        if self.data.len() > 0 {
            let byte = self.data[0] as u8;     
            if byte >= Encoding::EOL as u8 {
                self.ok = false;
            }else{
                self.encoding = match byte {
                    0x00 => { Encoding::Raw },
                    0x01 => { Encoding::RLEFixed },
                    _ => { Encoding::EOL }
                };
                self.pos = 8;
                self.ok = true;
            }
        }else{
            self.encoding = Encoding::Raw;
            self.ok = false;
        }
    }
}
