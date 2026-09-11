use crate::node::*;
use crate::media::*;

pub use u8 as PageIdx;

pub struct Page<'a> {        
    pub source: u8,
    pub id: &'a str,
    pub content: &'a [u8]    
}
impl Default for Page<'_> {
    fn default() -> Self { Page { source: 255, id: "", content: b"" } }
}

impl Media {
    pub fn get_page(&self, page_idx: PageIdx) -> Page<'_> {            
        let mut lib_idx = self.pages.len() as u8;
        for lib in self.pages.iter().rev() {
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
                            let content_len= reader.read_u16() as usize;                                                                                    
                            if reader.available() >= content_len {                                
                                if idx == page_idx {                                                                            
                                    let pos = reader.get_pos();
                                    //let content = str::from_utf8().unwrap_or("<utf8-error>");                                    
                                    return Page {
                                        source: lib_idx,                                                                        
                                        id: id,
                                        content: &data[pos .. pos + content_len]
                                    }
                                }                            
                            }
                            reader.set_pos(reader.get_pos() + content_len);                
                        }else{
                            break;
                        }                    
                    }
                }else{
                    break;
                }
            }            
        }

        return Page::default();
    }

}

#[repr(u8)]
pub enum PageInstruction {    
    DOM = 0,
    NodeContent = 1,
    NodeDirection = 2,
    NodeAlign = 3,    
    NodeWidth = 4,
    NodeHeight = 5,
    NodeVisible = 6,
    NodeMargin = 7,
    NodePadding = 8,
    NodeBackground = 9,
    NodeColor = 10,
    NodeBorder = 11,
    NodeFont = 12,
    NodeFrame = 13,
}

#[repr(u8)]
pub enum PageDOMInstruction {
    DocumentStart = 0,
    DocumentEnd = 1,
    NodeStart = 2,  
    NodeEnd = 3
}

#[cfg(all(feature = "xml", feature = "std"))]
use bytes::{BufMut, BytesMut, Bytes};
#[cfg(all(feature = "xml", feature = "std"))]
use crate::xmlparser::*;
#[cfg(all(feature = "xml", feature = "std"))]
impl Media {
    pub fn xml_to_bin (xml: &str) -> Bytes {           
        let mut page = BytesMut::new();

        log::debug!("converting xml (no_std)");

        let mut reader = XmlParser::new();

        page.put_u8(PageInstruction::DOM as u8 | ((PageDOMInstruction::DocumentStart as u8) << 4)); 
        loop {                              
            let event = reader.next(xml);        
            if event.is_err() {
                log::error!("xml error at {}: {}", reader.position(), event.err().unwrap());
                page.clear();
                break;
            }
            let position = reader.position();
            match event.unwrap() {
                XmlEvent::StartElement { name: _ } => {                                                              
                    page.put_u8(PageInstruction::DOM as u8 | ((PageDOMInstruction::NodeStart as u8) << 4));                  
                }
                XmlEvent::Attribute { key, value} => {                                         
                    match key {                        
                        "vertical"=>{
                            page.put_u8(PageInstruction::NodeDirection as u8 | 1 << 4);
                        }
                        "horizontal"=>{
                            page.put_u8(PageInstruction::NodeDirection as u8 | 0 << 4);
                        }
                        "width"=>{                                                                                    
                            if value.starts_with("flex"){
                                let num = value[4..].parse::<SizeW>().unwrap_or(1);                                
                                page.put_u8(PageInstruction::NodeWidth as u8);
                                page.put_u16_ne((WIDTH_FLEX - core::cmp::max(1 , core::cmp::min(num, 4)) + 1) as u16);
                            } else {
                                let res = value.parse::<SizeW>();
                                if res.is_ok() {
                                    page.put_u8(PageInstruction::NodeWidth as u8);
                                    page.put_u16_ne(res.unwrap());
                                }else{
                                    log::error!("illegal width \"{value}\"");
                                }
                            }                            
                        }
                        "height"=>{                            
                            if value.starts_with("flex"){                                
                                let num = value[4..].parse::<SizeH>().unwrap_or(1);                                                               
                                page.put_u8(PageInstruction::NodeHeight as u8);
                                page.put_u16_ne((HEIGHT_FLEX - core::cmp::max(1 , core::cmp::min(num, 4)) + 1) as u16);
                            } else {
                                let res = value.parse::<SizeW>();
                                if res.is_ok() {
                                    page.put_u8(PageInstruction::NodeHeight as u8);
                                    page.put_u16_ne(res.unwrap());
                                }else{
                                    log::error!("illegal height \"{value}\"");
                                }
                            }      
                        }
                        "align"=>{
                            match value {
                                "start"   => { page.put_u8(PageInstruction::NodeAlign as u8 | 0 << 4); },
                                "center"  => { page.put_u8(PageInstruction::NodeAlign as u8 | 1 << 4); },
                                "end"     => { page.put_u8(PageInstruction::NodeAlign as u8 | 2 << 4); },
                                "stretch" => { page.put_u8(PageInstruction::NodeAlign as u8 | 3 << 4); },
                                _=> { log::warn!("illegal align \"{value}\" ignored"); }
                            }                                          
                        }                        
                        "visible"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );
                            page.put_u8((PageInstruction::NodeVisible as u8) | ((value as u8 & 0xf) << 4));                        
                        }
                        "margin"=>{  
                            let values = control::decode_values(value);
                            page.put_u8(PageInstruction::NodeMargin as u8);                        
                            for value in values { page.put_i8(value); }
                        }
                        "padding"=>{                                                    
                            let values = control::decode_values(value);
                            page.put_u8(PageInstruction::NodePadding as u8);                        
                            for value in values { page.put_i8(value); }
                        }                        
                        "background"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );
                            page.put_u8(PageInstruction::NodeBackground as u8 | ((value as u8 & 0xf) << 4));                                                                            
                        }
                        "color"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 1; } );   
                            page.put_u8(PageInstruction::NodeColor as u8 | ((value as u8 & 0xf) << 4));                                                                                                                                                         
                        }
                        "border"|"frame"=>{        
                            let instruction = if key == "border" { PageInstruction::NodeBorder } else { PageInstruction::NodeFrame } as u8;
                            if value.eq_ignore_ascii_case("solid") {
                                page.put_u8(instruction | (BORDER_RECT << 4));                                                                            
                            }else{
                                let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 1; } );                                                                                    
                                page.put_u8(instruction | ((value as u8 & 0xf) << 4));                                                                            
                            }

                        }
                        "font"=>{                                          
                            let value: FontIdx = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } ); 
                            page.put_u8(PageInstruction::NodeFont as u8 | ((value as u8 & 0xf) << 4));                                                                                                                                                     
                        }
                        _=>{}
                    }                     
                },               
                XmlEvent::EndElement { name } => {                                         
                    log::debug!("{position:4}| </{name}>");
                    page.put_u8(PageInstruction::DOM as u8 | ((PageDOMInstruction::NodeEnd as u8) << 4));                  
                },
                XmlEvent::CData { data } => {                        
                    let str = data;
                    if str.len() > 0 {
                        log::debug!("{position:4}| Content: \"{data}\"");                    
                        page.put_u8(PageInstruction::NodeContent as u8);            
                        page.put(data.as_bytes());  
                        page.put_u8(0);                                    
                    }
                },            
                XmlEvent::EndDocument {} => {
                    log::debug!("{position:4}| end");                    
                    page.put_u8(PageInstruction:: DOM as u8 | ((PageDOMInstruction::DocumentEnd as u8) << 4)); 
                    break;          
                }
                _ => {}
            }
        }  

        Bytes::from(page)
    }
}