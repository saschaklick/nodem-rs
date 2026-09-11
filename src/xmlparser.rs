pub enum XmlEvent <'a> {        
    StartDocument {},
    EndDocument   {},
    StartElement  { name: &'a str },
    EndElement    { name: &'a str },
    Attribute     { key: &'a str, value: &'a str },
    CData         { data: &'a str }
}

pub struct XmlParserConfig {
    pub trim_content: bool
}

pub struct XmlParser {    
    position: usize,
    state: usize,
    mark: usize,
    mark_end: usize,
    mark_value: usize,
    config: XmlParserConfig
}

impl XmlParser {
    
    pub fn new() -> Self {
        return Self { position: 0, state: 0, mark: 0, mark_end: 0, mark_value: 0, config: XmlParserConfig { trim_content: true } };
    }

    pub fn next <'a>(&mut self, xml: &'a str) -> Result<XmlEvent<'a>, &'static str> {        
        match self.state {
            0 => {
                self.state = 1;
                self.mark = self.position;
                return Ok(XmlEvent::StartDocument {});
            }
            _ => {
                loop {
                    if xml.len() <= self.position {
                        return Ok(XmlEvent::EndDocument {});   
                    }
                    let character: char = xml.chars().nth(self.position).unwrap();
                    //log::info!("{} at {} [s: {}]", character, self.position, self.state);
                    match self.state {
                        1 => {
                            match character {                                
                                '<'  => {                                    
                                    self.state = 22;
                                    self.mark = self.position + 1;                                    
                                }
                                _  => {}
                            }                            
                        }
                        11 => {
                            match character {                                
                                '<'  => {
                                    let mark = self.mark + 1;
                                    self.state = 22;
                                    self.mark = self.position + 1;
                                    if mark != self.position {
                                        self.position += 1;
                                        let data = &xml[mark..self.position - 1];
                                        return Ok(XmlEvent::CData { data: if self.config.trim_content { data.trim() } else { data } });                                        
                                    }
                                }
                                _  => {}
                            }                            
                        }
                        22 => {
                            match character {
                                '/' => { self.state = 23; self.mark = self.position + 1; }
                                '!' => { self.state = 30; }
                                _ => { self.state = 2; continue; }
                            }
                        }
                        23 => {
                            match character {
                                ' '|'\t'|'\n'|'>' => {
                                    let mark = self.mark;
                                    self.state = if character == '>' { 1 } else { 24 };                                    
                                    self.mark = self.position;
                                    self.position += 1;                                                                        
                                    return Ok(XmlEvent::EndElement { name: &xml[mark..self.position - 1] });                                        
                                }
                                _ => {}
                            }
                        }
                        24 => { 
                            match character {
                                '>' => {
                                    let mark = self.mark;
                                    self.state = 1;           
                                    self.mark = self.position;                         
                                    self.position += 1;                                                                        
                                    return Ok(XmlEvent::EndElement { name: &xml[mark..self.position - 1] });                                        
                                }
                                _ => {}
                            }
                        }
                        30 => {
                            match character {
                                '-' => { self.state = 31; }
                                _ => { return Err("illegal character"); }
                            }
                        }
                        31 => {
                            match character {
                                '-' => { self.state = 32; }
                                _ => { return Err("illegal character"); }
                            }
                        }
                        32 => {
                            match character {
                                '-' => { self.state = 33; }
                                _ => {}
                            }
                        }
                        33 => {
                            match character {
                                '-' => { self.state = 34; }
                                _ => { self.state = 32; }
                            }
                        }
                        34 => {
                            match character {
                                '>' => { self.state = 1; self.mark = self.position; }
                                _ => { self.state = 32; }
                            }
                        }
                        2 => {
                            match character {
                                ' '|'\t'|'\n' => {
                                    self.state = 3;                                    
                                    self.position += 1;                                    
                                    return Ok(XmlEvent::StartElement { name: &xml[self.mark..self.position - 1] });                                        
                                }
                                '>' => {
                                    let mark = self.mark;
                                    self.state = 11;
                                    self.mark = self.position;
                                    self.position += 1;
                                    return Ok(XmlEvent::StartElement { name: &xml[mark..self.position - 1] });                                        
                                }
                                _ => {}
                            }
                        }
                        3 => {
                            match character {
                                '>' => {
                                    self.state = 11;
                                    self.mark = self.position;
                                    self.position += 1;
                                    return Ok(XmlEvent::StartElement { name: &xml[self.mark..self.position] });    
                                }
                                'a'..'z'|'A'..'Z'|'0'..'9'|'_' => {
                                    self.mark = self.position;
                                    self.state = 4;
                                }
                                _ => {}
                            }
                        }
                        36 => {
                            match character {
                                '>' => {
                                    self.state = 11;
                                    self.mark = self.position;                                    
                                }
                                'a'..'z'|'A'..'Z'|'0'..'9'|'_' => {
                                    self.mark = self.position;
                                    self.state = 4;
                                }
                                _ => {}
                            }
                        }
                        4 => {
                            match character {
                                '>' => {
                                    self.state = 36;                                         
                                    return Ok(XmlEvent::Attribute { key: &xml[self.mark..self.position], value: "" });    
                                }
                                ' ' => {
                                    self.state = 3;
                                    self.position += 1;
                                    return Ok(XmlEvent::Attribute { key: &xml[self.mark..self.position - 1], value: "" });    
                                }
                                '=' => {
                                    self.state = 5;
                                    self.mark_end = self.position;   
                                    self.mark_value = self.position + 1;                              
                                }
                                'a'..'z'|'A'..'Z'|'0'..'9'|'_' => {}
                                _ => {
                                    return Err("illegal character in attribute name");
                                }
                            }
                        }
                        5 => {
                            match character {                                
                                '"' => {
                                    self.mark_value = self.position + 1;
                                    self.state = 7;                                                                        
                                }
                                '>' => {
                                    self.state = 4;     
                                    return Ok(XmlEvent::Attribute { key: &xml[self.mark..self.mark_end], value: "" });    
                                }                                                           
                                _ => { self.state = 6; }
                            }
                        }
                        6 => {
                            match character {                                
                                ' '|'\t'|'\n'|'>' => {
                                    let mark = self.mark;
                                    match character {
                                        '>' => { self.state = 11; self.mark = self.position; }
                                        _ => { self.state = 36; }
                                    }                                    
                                    self.position += 1;
                                    return Ok(XmlEvent::Attribute { key: &xml[mark..self.mark_end], value: &xml[self.mark_value..self.position - 1] });    
                                }                                                                                                
                                _ => {}                            
                            }
                        }
                        7 => {
                            match character {                                
                                '"' => {
                                    self.state = 36;                                                                                                        
                                    self.position += 1;
                                    return Ok(XmlEvent::Attribute { key: &xml[self.mark..self.mark_end], value: &xml[self.mark_value..self.position - 1] });                                          
                                }                                                                
                                _ => {}
                            }
                        }
                        _=> {                            
                            return Err("invalid state");
                        }
                    }
                    self.position += 1;
                }
            }            
        }        
    }

    pub fn position(&self) -> usize {
        return self.position;
    }
}