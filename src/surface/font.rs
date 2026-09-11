
use crate::surface::*;
use crate::stream::*;
use crate::media::image::*;
use crate::media::font::*;

fn apply_drawing(drawing: &mut media::image::Settings, option:u8) {
    match option {
        0x00 => { drawing.use_mask = false; }
        0x01 => { drawing.use_mask = true; }
        0x02 => { drawing.inverted = false; }
        0x03 => { drawing.inverted = true; }
        _ => { log::error!("unsupported drawing option {}", option); }
    }
}

fn apply_typesetting(typesetting: &mut media::font::Settings, option:u8, value: u8) {
    match option {
        0x00 => typesetting.mono_spaced = Alignment::Proportional,
        0x01 => typesetting.mono_spaced = Alignment::Start,
        0x02 => typesetting.mono_spaced = Alignment::Center,
        0x03 => typesetting.mono_spaced = Alignment::End,        
        0x04 => typesetting.inverted = false,
        0x05 => typesetting.inverted = true,
        0x06 => typesetting.fixed_height = false,
        0x07 => typesetting.fixed_height = true,
        0x08 => typesetting.space_width = value,        
        0x09 => typesetting.line_spacing = value,
        0x0A => typesetting.glyph_spacing = value,
        _ => { log::error!("unsupported typesetting option {}", option); }
    }    
}

impl Surface {        
    pub fn get_text_size(&self, ident: Identifier, text: &str) -> Size {
        return self._process_text(ident, text, None);
    }

    pub fn draw_text(&self, ident: Identifier, text: &str, position: Point) {
        self._process_text(ident, text, Some(position));
    }

    pub fn _process_text(&self, ident: Identifier, text: &str, position: Option<Point>) -> Size {
        let mut font = self.media.get_font(ident);    
        let mut drawing = self.media.drawing;
        let mut typesetting = self.media.typesetting;
        let mut cursor = position.unwrap_or(Point{ x: 0, y: 0 }).clone();
        let mut size = Size::default();          
        let mut rem_text = &text[0 ..];
        let mut line_height: PosY = 0;        
        let mut instr_str: Option<&str> = None;
        let mut value_str: Option<&str> = None;
        let mut instr_len: usize = 0;
        let mut instruction = [0u8;5];
        loop {            
            let mut chrs = rem_text.chars();
            let chr_res = chrs.next();            
            if chr_res.is_none() {
                break;            
            }else{                
                let mut chr = chr_res.unwrap();

                if instr_str.is_some() {                                              
                    if chr == '}' {
                        let instr = &instr_str.take().unwrap()[ .. instr_len];                                                                        
                        let mut parts = instr.split(":");
                        let name = parts.next().unwrap_or("").trim();
                        value_str = parts.next();
                        match name {
                            "nl" | "br" => {
                                instruction[0] = 0;
                                chr = '\n';
                            }
                            "img" => {
                                instruction[0] = crate::surface::IMAGE as u8;                                
                                instruction[1] = value_str.unwrap_or("0").trim().parse::<u8>().unwrap_or(u8::MAX);                                
                            }
                            "font" => {
                                instruction[0] = crate::surface::FONT as u8;
                                instruction[1] = value_str.unwrap_or("0").trim().parse::<u8>().unwrap_or(u8::MAX);
                            }
                            "mons" | "mono" | "mone"| "prop"| "tnrm" | "tinv" | "fixh" | "varh" | "spac" | "line" | "kern" => {
                                instruction[0] = crate::surface::TYPESETTING as u8;
                                instruction[1] = match name {
                                    "prop" => 0x00,
                                    "mons" => 0x01,
                                    "mono" => 0x02,
                                    "mone" => 0x03,
                                    "tnrm" => 0x04,
                                    "tinv" => 0x05,
                                    "fixh" => 0x06,
                                    "varh" => 0x07,                                    
                                    "spac" => 0x08,
                                    "line" => 0x09,
                                    "kern" => 0x0A,
                                    _      => u8::MAX
                                };
                                instruction[2] = value_str.unwrap_or("0").trim().parse::<u8>().unwrap_or(u8::MAX);
                            }
                            "flat" | "mask" | "nrm" | "inv" => {
                                instruction[0] = crate::surface::DRAWING as u8;
                                instruction[1] = match name {
                                    "flat" => 0,
                                    "mask" => 1,
                                    "nrm"  => 2,
                                    "inv"  => 3,
                                    _      => u8::MAX
                                }                                
                            }
                            _ => {
                                instruction[0] = u8::MAX;                                
                                log::warn!("unknown instruction \"{}\"", name);
                            }
                        }                                                                 
                    }else{
                        instr_len += 1;
                    }
                }                               

                if instruction[0] == 0 && instr_str.is_none() {
                    if typesetting.fixed_height {
                        line_height = font.full_height as PosY;
                    }

                    match chr {
                        '\x00'      => { break; }
                        crate::surface::IMAGE | crate::surface::FONT | crate::surface::DRAWING | crate::surface::TYPESETTING => {
                            let next_res = chrs.next();
                            if next_res.is_none(){ break; }                            
                            instruction[0] =  chr as u8;
                            instruction[1] =  next_res.unwrap() as u8;                            
                        }                       
                        '\n'        => {
                            cursor.y += (line_height as PosY).checked_add(typesetting.line_spacing as PosY).unwrap_or(PosY::MAX);
                            cursor.x = if position.is_some() { position.unwrap().x } else { 0 };
                            line_height = 0;                             
                        } 
                        ' '         => {
                            cursor.x += match typesetting.mono_spaced {
                                Alignment::Proportional => { typesetting.space_width as PosX }
                                _ => { font.mono_width as PosX }
                            }.saturating_add(typesetting.glyph_spacing as PosX);                                         
                            line_height = font.base_height as PosY;                            
                        }                        
                        _ => {
                            if chr == '{' {
                                chr = chrs.next().unwrap_or('{');
                                if chr != '{' {                                    
                                    instr_str = Some(&rem_text[1..]);                                    
                                    rem_text = &rem_text[1..];
                                    instr_len = 0;                                    
                                    continue;
                                }
                            }

                            let glyph = self.media.get_glyph(&font, chr);                        
                            let mut reader = Stream::new(glyph);
                            if reader.available() >= 1 {
                                let width = reader.read_u8();                    
                                if width == 0 && typesetting.placeholders == true {  
                                    let color_no = if typesetting.inverted { 0 } else { 1 };
                                    let color = self.palette[color_no as usize];
                                    if position.is_some() {
                                        self.draw_rect(Area { point: cursor, size: Size { width: font.mono_width as SizeW, height: font.base_height as SizeH } }, color);                                        
                                    }
                                    cursor.x += (font.mono_width as PosX).checked_add(typesetting.glyph_spacing as PosX).unwrap_or(PosX::MAX);
                                }else{
                                    if reader.available() >= GLYPH_HEADER_SIZE - 1 {
                                        let height = reader.read_u8();
                                        let mut x_offset = reader.read_i8();
                                        let mut y_offset = reader.read_i8();                            
                                        let pos = reader.get_pos();                                                        
                                        
                                        x_offset = match typesetting.mono_spaced {
                                            Alignment::Proportional => { x_offset }
                                            Alignment::Start =>  { x_offset }
                                            Alignment::Center => { x_offset.checked_add(font.mono_width.checked_sub(width).unwrap_or(0).checked_div(2).unwrap_or(0) as i8).unwrap_or(x_offset) }
                                            Alignment::End =>    { x_offset.checked_add(font.mono_width.checked_sub(width).unwrap_or(0) as i8).unwrap_or(x_offset) }
                                        };
                                        if typesetting.fixed_height {
                                            y_offset = y_offset.checked_add(font.full_height.checked_sub(height).unwrap_or(0).checked_div(2).unwrap_or(0) as i8).unwrap_or(y_offset);
                                        }

                                        if position.is_some() {
                                            let position = Point {
                                                x: (cursor.x as PosX).checked_add(x_offset as PosX).unwrap_or(PosX::MAX),
                                                y: (cursor.y as PosY).checked_add(y_offset as PosY).unwrap_or(PosY::MAX)
                                            };                            
                                            
                                            Surface::render_stream(self, ImageType::Indexed1 as u8, width, height, &glyph[pos ..], position, typesetting.inverted, 0);
                                            //self.draw_rect(Area { point: position, size: Size { width: width as SizeW, height: height as SizeH } }, 1);                         
                                        }
                                                                    
                                        cursor.x += match typesetting.mono_spaced {
                                            Alignment::Proportional => { (width as PosX).saturating_add(x_offset as PosX) },
                                            _ => { font.mono_width as PosX }                                                                                    
                                        }.saturating_add(typesetting.glyph_spacing as PosX);                
                                        if line_height < height as PosY + y_offset as PosY {
                                            line_height = height as PosY + y_offset as PosY;
                                        }
                                    }
                                }
                            }                        
                        }
                    }
                }

                if instruction[0] != 0 {
                    match instruction[0] as char {
                        crate::surface::IMAGE => {                            
                            let image_idx = instruction[1] as u8;                    
                            let ident = if value_str.is_some() {
                                Identifier::Both(image_idx, value_str.unwrap())
                            } else {
                                Identifier::Index(image_idx)
                            };
                            let image_size = if position.is_some() {
                                self.draw_image(ident, cursor, Some(drawing))                                
                            } else {
                                self.get_image_size(ident)
                            };
                            cursor.x = cursor.x.saturating_add(image_size.width as PosX);
                            cursor.x = cursor.x.saturating_add(typesetting.glyph_spacing as PosX);                                            
                            if line_height < image_size.height as PosY {
                                line_height = image_size.height as PosY;
                            }                            
                        }
                        crate::surface::FONT        => {
                            font = if value_str.is_some() {
                                self.media.get_font(Identifier::Both(instruction[1] as u8, value_str.unwrap()))
                            }else{
                                self.media.get_font(Identifier::Index(instruction[1] as u8))
                            };
                        }
                        crate::surface::DRAWING     => { apply_drawing(&mut drawing, instruction[1] as u8); }
                        crate::surface::TYPESETTING => { apply_typesetting(&mut typesetting, instruction[1] as u8, instruction[2] as u8); }                    
                        _ => {}
                    }
                    instruction[0] = 0;
                }                
                size.width = cmp::max(size.width as PosX, cursor.x.saturating_sub(typesetting.glyph_spacing as PosX)) as SizeW;            
            }  
            rem_text = &rem_text[1..];
        }
        size.height = cursor.y.saturating_add(line_height as PosY) as SizeW;        
        return size;
    }
}