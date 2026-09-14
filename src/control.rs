use crate::{ surface::Surface, media::{ Media } };

mod surface;
mod media;
#[cfg(feature="dom")]
mod dom;
#[cfg(feature="virtmach")]
mod program;

#[derive(PartialEq)]
pub enum ControlMode {
    LineMode,
    PKGMode        
}

#[repr(u8)]
enum Ret {
    Ok = 0,        
    UnknownCommand = 1 
}

#[repr(u8)]
#[derive(Clone)]
#[derive(Copy)]
pub enum LoaderRet {
    Ok = 0,
    CTR = 1,
    Aborted = 2,
    NotEnoughSpace = 3,
    PKGFailed = 4
}

struct Loader {
    state: u8,
    size: usize,
    position: usize,    
    error: LoaderRet
}

pub struct Control {
    pub mode: ControlMode,
    loader: Loader
}
impl Default for Control {
    fn default() -> Self { Self {
        mode : ControlMode::LineMode,
        loader : Loader { state: 0, size: 0, position: 0, error: LoaderRet::Ok }
    } }
}

impl Control {
    pub fn is_loader_busy(&self) -> bool {
        match self.loader.error {
            LoaderRet::Ok => self.loader.state != 0,
            _ => true
        }         
    }
    
    pub fn get_loader_cts(&self) -> bool {
        self.loader.state == 4 && self.loader.position == 0
    }
    
    pub fn get_loader_progress(&self, range: usize) -> Option<usize> {
        if self.loader.state == 0 { None } else { Some(range * self.loader.position / self.loader.size) }
    }
    
    pub fn get_loader_error(&self) -> LoaderRet {
        self.loader.error
    }

    pub fn abort_loader(&mut self) {
        self.mode = ControlMode::LineMode;
        self.loader.state = 0;
        self.loader.position = 0;
        self.loader.size = 0;
        self.loader.error = LoaderRet::Aborted;
    }

    pub fn process <'a>(&mut self, buf: &[u8], surface: &mut Surface, listeners: &mut [Option<&mut dyn IControl>;4], res: &mut dyn core::fmt::Write) -> (usize, core::fmt::Result) {
        let buf = buf;   
        let mut buf_pos = 0usize;
        while buf_pos < buf.len() {                                    
            //for byte in buf { log::info!("{:02x} {}", byte, if *byte > 0x20 && *byte < 0x7f { unsafe { char::from_u32_unchecked(*byte as u32) } } else { '.' });            }
            match self.mode {
                ControlMode::LineMode => {                        
                    let line_pos = buf[buf_pos..].iter().position(|a| *a == b'\r' || *a == b'\n');                        
                    if line_pos.is_some() {
                        let line = str::from_utf8(&buf[buf_pos..buf_pos + line_pos.unwrap()]).unwrap_or("");                            
                        
                        buf_pos += line_pos.unwrap() + 1;                            
                        
                        match line {
                            "info" => {                                  
                                res.write_str(concat!(env!("CARGO_PKG_NAME"), ",", env!("NODEM_ARCH"), ",", env!("CARGO_PKG_VERSION"), ",\"COPYRIGHT (c) 2026 BY ", env!("CARGO_PKG_AUTHORS"), "\"\r\n")).expect("");
                                #[cfg(feature = "std")]
                                res.write_str("std").expect("");                                        
                                #[cfg(not(feature = "std"))]
                                res.write_str("core").expect("");                                           
                                res.write_str(",control").expect("");                                        
                                #[cfg(feature = "inspect")]                             
                                res.write_str(",inspect").expect("");                                                                                                                                                                  
                                #[cfg(feature = "dom")]
                                res.write_str(",dom").expect("");                                        
                                #[cfg(all(feature = "dom", feature = "xml"))]
                                res.write_str(",xml").expect("");                                        
                                #[cfg(feature = "ninepatch")]
                                res.write_str(",ninepatch").expect("");
                                #[cfg(feature = "vm")]
                                res.write_str(",vm").expect("");                                        
                                #[cfg(all(feature = "vm", feature = "compile"))]
                                res.write_str(",compile").expect("");                                        
                                res.write_str("\r\n").expect("");                                
                                Control::send_result("", Ret::Ok as u8, res).0;
                            },
                            "pkg" => {
                                self.mode = ControlMode::PKGMode;
                                self.loader.position = 0;
                                self.loader.size = 0;  
                                log::info!("loader init");                                      
                            }
                            _ => {                                
                                let mut ret = surface.process_line(line, &Media::default(), res);
                                if !ret.0  {                                    
                                    for target in &mut * listeners {
                                        if target.is_some() {                                                                     
                                            ret = target.as_mut().unwrap().process_line(line, &surface.media, res);
                                            if ret.0 {                                            
                                                break;
                                            }
                                        }
                                    }                                                             
                                }                                                                                                             
                                if !ret.0 {
                                    Control::send_result("", Ret::UnknownCommand as u8, res).0;
                                }                                                                
                            }
                        }                        
                    }else{
                        return (buf_pos, Ok(()));
                    }
                },
                ControlMode::PKGMode => {     
                    let mut listener: &mut dyn IControlLoader = surface.get_loader().unwrap();
                    for target in &mut * listeners {
                        if target.is_some() {                                                                     
                            let ret = target.as_mut().unwrap().get_loader();
                            if ret.is_some() {
                                listener = ret.unwrap();
                                break;
                            }
                        }
                    }                                                             
                    while buf_pos < buf.len() {                                            
                        let byte = buf[buf_pos];
                        buf_pos += 1;                        
                        match self.loader.state {                    
                            0..=2 => { self.loader.size += (byte as usize) << 8 * self.loader.state; self.loader.state += 1; },                            
                            3 => {
                                if listener.process_loader_start(self.loader.size) > 0 { 
                                    self.loader.size += (byte as usize) << 24;
                                    self.loader.state = 4;                                                                                                          
                                    log::info!("loader start: {}b", self.loader.size);
                                    Control::send_result("pkg", LoaderRet::CTR as u8, res).1.expect("");
                                }else{
                                    self.loader.error = LoaderRet::NotEnoughSpace;
                                    Control::send_result("pkg", LoaderRet::NotEnoughSpace as u8, res).1.expect("");
                                }
                            },
                            4 => {                                
                                listener.process_loader_data(&[byte;1], self.loader.position);                                            
                                self.loader.position += 1;
                                match self.loader.size - self.loader.position {                                    
                                    0 => {
                                        log::info!("loader end: {}b", self.loader.size);
                                        self.mode = ControlMode::LineMode;
                                        self.loader.state = 0;
                                        self.loader.size = 0;                                                                                                                                                                                                
                                        self.loader.error = listener.process_loader_end();                                                                                                                                                                        
                                        Control::send_result("pkg", self.loader.error as u8, res).1.expect("");
                                    }
                                    _ => {}
                                }                                                                                                                               
                            },
                            _ => {}
                        }                                      
                    }
                }            
            }            
        }     
        //log::info!("loader: {} {}", buf.len(), self.loader.position);       
        (buf_pos, Ok(()))
    }     

    pub fn send_result(prefix: &str, return_code: u8, res: &mut dyn core::fmt::Write) -> (bool, core::fmt::Result) {
        res.write_str(prefix).expect("");
        res.write_str(str::from_utf8(&[0x30 + return_code]).unwrap()).expect("");
        res.write_str("\r\n").expect("");
        (true, Ok(()))
    }     
}

pub trait IControl {
    fn process_line(&mut self, _line: &str, _media: &Media, res: &mut dyn core::fmt::Write) -> (bool, core::fmt::Result) { (false, res.write_str("?")) }
    
    fn get_loader(&mut self) -> Option<&mut dyn IControlLoader> { None }
}

pub trait IControlLoader {
    fn process_loader_start(&mut self, _len: usize) -> usize { 0 }
    
    fn process_loader_data(&mut self, _buf: &[u8], _pos: usize) {}
    
    fn process_loader_end(&mut self) -> LoaderRet { LoaderRet::Ok }    
}
impl IControl for dyn IControlLoader {
    fn get_loader(&mut self) -> Option<&mut dyn IControlLoader> { Some(self) }
}

pub fn decode_values<T: core::str::FromStr + core::fmt::Debug + core::clone::Clone + core::marker::Copy>(text: &str) -> [T;4] where <T as core::str::FromStr>::Err: core::fmt::Debug {    
    let mut ret: [T;4] = ["0".parse().unwrap();4];
    let mut split = text.split(" ");        
    split.next().map(|text|{ text.parse().map(|value| { ret.fill(value); }) });
    split.next().map(|text|{ text.parse().map(|value| { ret[1] = value; ret[3] = value; }) });
    split.next().map(|text|{ text.parse().map(|value| { ret[2] = value; }) });
    split.next().map(|text|{ text.parse().map(|value| { ret[3] = value; }) });    
    return ret;
}