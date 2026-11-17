use crate::{Color, ColorType};

#[repr(u8)]
#[derive(PartialEq)]
pub enum Encoding {
    Index1Raw      = 0x00,
    Index1RLEFixed = 0x01,
    Rgb888Raw      = 0x10,
    Rgb888RLEFixed = 0x11,
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

pub struct BitStream<'a> {
    data: &'a [u8],
    pub ok: bool,
    pos: usize,        
    rpt_cnt: usize,
    rpt_val: u8,
    raw_cnt: usize,
    encoding: Encoding
}
impl <'a> BitStream<'a> {
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
            Encoding::Index1Raw => {
                let ret = (self.data[self.pos / 8] >> (7 - (self.pos % 8))) & 1;
                self.pos += 1;
                ret
            },
            Encoding::Index1RLEFixed => {
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
                    0x00 => { Encoding::Index1Raw },
                    0x01 => { Encoding::Index1RLEFixed },
                    _ => { Encoding::EOL }
                };
                self.pos = 8;
                self.ok = true;
            }
        }else{
            self.encoding = Encoding::Index1Raw;
            self.ok = false;
        }
    }
}

pub struct PixelStream<'a> {
    encoding: Encoding,
    data: &'a [u8],
    palette: &'a [ColorType],
    pub ok: bool,
    pos: usize,  
    end: usize,        
    rpt_cnt: usize,
    rpt_val: u8,
    rpt_col: ColorType,
    raw_cnt: usize,    
}
impl <'a> PixelStream<'a> {
    pub fn new(data: &'a [u8], palette: &'a [ColorType]) -> Self {   
        let encoding = if data.len() == 0 {
            Encoding::EOL
        }else{            
            match data[0] {
                0x00 => Encoding::Index1Raw,
                0x01 => Encoding::Index1RLEFixed,
                0x10 => Encoding::Rgb888Raw,
                0x11 => Encoding::Rgb888RLEFixed,
                _ => Encoding::EOL                
            }
        };                
        let data = match encoding { Encoding::EOL => &[], _ => &data[1..] };
        
        Self::with_encoding(encoding, data, palette)
    }    

    pub fn with_encoding(encoding: Encoding, data: &'a [u8], palette: &'a [ColorType]) -> Self {
        let end = match encoding {
            Encoding::Index1Raw | Encoding::Index1RLEFixed => data.len() * 8,
            Encoding::Rgb888Raw | Encoding::Rgb888RLEFixed => data.len(),            
            _ => data.len()
        };        

        Self { encoding: encoding, data: data, palette: palette, ok: true, pos: 0, end: end, rpt_cnt: 0, rpt_val: 0, rpt_col: Color::TRANSPARENT, raw_cnt: 0 }
    }      
    
    pub fn next(&mut self) -> ColorType {
        if self.pos >= self.end {
            self.ok = false;
            Color::TRANSPARENT
        }else{
            match self.encoding {
                Encoding::Index1Raw => {
                    let ret = ((self.data[self.pos / 8] >> (7 - (self.pos % 8))) & 1) as usize;
                    self.pos += 1;                
                    if ret < self.palette.len() { self.palette[ret] } else { Color::TRANSPARENT }                
                },
                Encoding::Index1RLEFixed => {
                    if self.rpt_cnt == 0 && self.raw_cnt == 0 {
                        if (self.data[self.pos / 8] >> 7) & 1 == 0 {
                            self.raw_cnt = 7;                                            
                        } else {
                            self.rpt_val = (self.data[self.pos / 8] >> 6) & 1;
                            self.rpt_cnt = (self.data[self.pos / 8] & 0b00111111) as usize;                                               
                        }
                    }
                    let color_no = if self.rpt_cnt > 0 {
                        self.rpt_cnt -= 1;
                        if self.rpt_cnt == 0 { self.pos += 8; }
                        self.rpt_val                
                    } else {                    
                        let ret = self.data[self.pos / 8] >> (self.pos % 8) & 1;
                        self.pos += 1;
                        self.raw_cnt -= 1;
                        if self.raw_cnt == 0 { self.pos += 1; }                    
                        ret
                    } as usize;
                    if color_no < self.palette.len() { self.palette[color_no] } else { Color::TRANSPARENT }
                },
                Encoding::Rgb888Raw => {                
                    let pos = self.pos * 3;
                    let color = Color::from_rgb(self.data[pos], self.data[pos + 1], self.data[pos + 2]);
                    self.pos += 1;
                    color                
                },
                Encoding::Rgb888RLEFixed => {
                    if self.rpt_cnt == 0 && self.raw_cnt == 0 {
                        let hdr = self.data[self.pos];
                        let cnt = (hdr & 0x7f) as usize + 1;
                        if hdr & 0x80 == 0 {
                            self.raw_cnt = cnt;
                            self.pos += 1;                       
                        } else {
                            self.rpt_col = Color::from_rgb(self.data[self.pos + 1], self.data[self.pos + 2], self.data[self.pos + 3]);
                            self.rpt_cnt = cnt;
                            self.pos += 4;
                        }
                    }
                    if self.rpt_cnt > 0 {
                        self.rpt_cnt -= 1;
                        self.rpt_col
                    } else {
                        let ret = Color::from_rgb(self.data[self.pos], self.data[self.pos + 1], self.data[self.pos + 2]);
                        self.raw_cnt -= 1;
                        self.pos += 3;
                        ret
                    }
                },
                _ => { Color::TRANSPARENT }
            }
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.rpt_cnt = 0;
        self.raw_cnt = 0;
        self.ok = match self.encoding {
            Encoding::EOL => false,
            _ => true
        };        
    }
}

