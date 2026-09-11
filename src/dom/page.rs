use log;
#[cfg(feature = "alloc")]
use nostd::alloc::{ realloc, Layout };

use crate::dom::{ *, Ret };
use crate::node::Alignment;
use crate::stream::Stream;

#[cfg(all(feature = "dom", feature = "xml"))]
use crate::xmlparser::*;

use crate::media::page::{PageInstruction, PageDOMInstruction};

impl DOM {
    pub fn from (&mut self, bin: &[u8]) -> Ret {        
        let mut err = Ret::Ok;
        let mut reader = Stream::new(bin);
        let dom = self;
        let mut depth = 0;
        let mut p_stack = [NODE_MAX as u8; DOM_DEPTH_MAX];
        let mut n_i = 0;

        dom.reset();
        
        log::debug!("parsing bin");
                
        while reader.available() > 0 {                        
            let node = &mut dom.nodes[n_i as usize];            

            fn values(reader: &mut Stream) -> [i8;4] {
                [0i8;4].map(|_| { reader.read_i8() as Offset })
            }
            
            let next = reader.read_u8();
            let instruction = next & 0xf;
            let value = next >> 4;
            let position = reader.get_pos();
            let instruction = PageInstruction::from(match instruction {
                0 => PageInstruction::DOM,
                1 => PageInstruction::NodeContent,
                2 => PageInstruction::NodeDirection,
                3 => PageInstruction::NodeAlign,                
                4 => PageInstruction::NodeWidth,
                5 => PageInstruction::NodeHeight,
                6 => PageInstruction::NodeVisible,
                7 => PageInstruction::NodeMargin,
                8 => PageInstruction::NodePadding,
                9 => PageInstruction::NodeBackground,
                10 => PageInstruction::NodeColor,
                11 => PageInstruction::NodeBorder,
                12 => PageInstruction::NodeFont,
                13 => PageInstruction::NodeFrame,
                _ => { err = Ret::InvalidDOMInstruction; log::error!("invalid dom instruction {}:{:02x}", position, bin[position]); continue; }
            });
            match instruction {
                PageInstruction::DOM => match PageDOMInstruction::from( match value {
                    0 => PageDOMInstruction::DocumentStart,
                    1 => PageDOMInstruction::DocumentEnd,
                    2 => PageDOMInstruction::NodeStart,  
                    3 => PageDOMInstruction::NodeEnd,
                    _ => { err = Ret::InvalidDOMInstruction; log::error!("invalid dom instruction {}:{:02x}", position, bin[position]); continue; }
                }) {
                    PageDOMInstruction::NodeStart => {                             
                        if depth > 0 {
                            n_i += 1;                    
                            let c_i = dom.nodes[p_stack[depth - 1] as usize].first_child;
                            if c_i == NODE_MAX {
                                dom.set_child(p_stack[depth - 1], n_i);
                            }else{
                                let mut s_i = c_i; 
                                loop {
                                    let ns_i = dom.nodes[s_i as usize].next_sibling;
                                    if ns_i == NODE_MAX {
                                        dom.nodes[s_i as usize].next_sibling = n_i;
                                        break;
                                    }else{
                                        s_i = ns_i;
                                    }
                                }
                            }
                        }                                  
                        log::trace!("{position:4}| ({depth}:{n_i}) <N>");     
                        p_stack[depth] = n_i;
                        depth += 1;     
                    },
                    PageDOMInstruction::NodeEnd => {                    
                        depth -= 1;
                        log::trace!("{position:4}| ({depth}:{n_i}) </N>");
                    },
                    PageDOMInstruction::DocumentStart => {
                        log::trace!("{position:4}| ({depth}:{n_i}) start");
                        
                    }, 
                    PageDOMInstruction::DocumentEnd => {
                        log::trace!("{position:4}| ({depth}:{n_i}) end");
                        break;
                    }                    
                },
                PageInstruction::NodeContent => {
                    let data_start= position;
                    let mut data_end = data_start;
                    while data_end < bin.len() && bin[data_end] != 0 {
                        data_end += 1;
                    }
                    let data = str::from_utf8(&bin[data_start..data_end]).unwrap_or("");
                    if data.len() > 0 {
                        let raw = data.as_ptr();                
                        if node.plot.content_idx == CONTENT_MAX && node.first_child == NODE_MAX {
                           dom.set_content(n_i, ContentContainer::Raw { raw: raw, len: data.len() });
                           log::trace!("{position:4}| ({depth}:{n_i}) Content: \"{data}\"");                        
                        }
                    }
                    reader.set_pos(data_end);
                },              
                PageInstruction::NodeDirection => {                    
                    node.set_direction(match value { 1 => Direction::Vertical, _ => Direction::Horizontal });
                    log::trace!("{position:4}| ({depth}:{n_i}) direction: {}", value);
                },
                PageInstruction::NodeAlign => {                    
                    node.set_alignment(match value { 1 => Alignment::Center, 2 => Alignment::End, 3 => Alignment::Stretch, _ => Alignment::Start });
                    log::trace!("{position:4}| ({depth}:{n_i}) align: {}", value);
                }                
                PageInstruction::NodeWidth => {                    
                    node.set_width(reader.read_u16() as SizeW);
                    log::trace!("{position:4}| ({depth}:{n_i}) width: {}", node.plot.size.width);                    
                },
                PageInstruction::NodeHeight => {                    
                    node.set_height(reader.read_u16() as SizeH);
                    log::trace!("{position:4}| ({depth}:{n_i}) height: {}", node.plot.size.height);                    
                },
                PageInstruction::NodeVisible => {                    
                    node.set_visibility(match value { 1 => Visibility::Visible, _ => Visibility::Hidden });
                    log::trace!("{position:4}| ({depth}:{n_i}) visibile: {}", value);
                },                
                PageInstruction::NodeMargin => {                    
                    let values = values(&mut reader);
                    dom.set_margin(n_i, values);                     
                    log::trace!("{position:4}| ({depth}:{n_i}) margin: {:?}", values);
                },
                PageInstruction::NodePadding => {                                        
                    let values = values(&mut reader);
                    dom.set_padding(n_i, values);                     
                    log::trace!("{position:4}| ({depth}:{n_i}) padding: {:?}", values);
                },
                PageInstruction::NodeBackground => {                    
                    dom.request_style(n_i).set_background(value); 
                    log::trace!("{position:4}| ({depth}:{n_i}) background: {}", value);
                },
                PageInstruction::NodeColor => {                    
                    dom.request_style(n_i).set_color(value); 
                    log::trace!("{position:4}| ({depth}:{n_i}) color: {}", value);
                },
                PageInstruction::NodeBorder | PageInstruction::NodeFrame => {                    
                    let framing = match instruction { PageInstruction::NodeBorder => true, _ => false };
                    node.set_framing(framing);
                    dom.request_style(n_i).set_border(value);                    
                    log::trace!("{position:4}| ({depth}:{n_i}) border: {} framing: {}", value, framing);
                },
                PageInstruction::NodeFont => {                    
                    dom.request_style(n_i).set_font(value); 
                    log::trace!("{position:4}| ({depth}:{n_i}) font: {}", value);
                }                           
            }            
        }    
        return err;            
    }

    #[cfg(not(feature = "xml"))]
    pub fn from_xml (&mut self, _xml: &str) -> Ret {       
        self.clear();
        log::error!("xml feature disabled");
        return Ret::NoAllocFeature;
    }

    #[cfg(all(feature = "dom", feature = "xml"))]
    pub fn from_xml(&mut self, xml: &str) -> Ret {        
        self.alloc_buf.raw = unsafe{ if self.alloc_buf.raw.is_null() {
                alloc(Layout::from_size_align(xml.len(), 4).unwrap())
            }else{
                realloc(self.alloc_buf.raw, Layout::from_size_align(self.alloc_buf.len, 4).unwrap(), xml.len())
            }
        };
        self.alloc_buf.len = xml.len();
        if self.alloc_buf.raw != core::ptr::null_mut() {
            unsafe{ self.alloc_buf.raw.copy_from(xml.as_ptr(), xml.len()) };                
        }else{
            log::error!("alloc failed");
            return Ret::AllocFailure;        
        }                                
        let xml = str::from_utf8(unsafe{ &core::slice::from_raw_parts(self.alloc_buf.raw, self.alloc_buf.len) }).unwrap_or("");        

        self.from_xml_static(xml)
    }

    #[cfg(all(feature = "dom", feature = "xml"))]
    pub fn from_xml_static (&mut self, xml: &'static str) -> Ret {        
        let mut err = Ret::Ok;
        let dom = self;
        let mut depth = 0;
        let mut p_stack = [NODE_MAX as u8; DOM_DEPTH_MAX];
        let mut n_i = 0;

        dom.clear();

        log::debug!("parsing xml (no_std)");

        let mut reader = XmlParser::new();

        loop {                              
            let event = reader.next(xml);        
            if event.is_err() {
                err = Ret::InvalidXML;
                log::error!("xml error at {}: {}", reader.position(), event.err().unwrap());
                break;
            }
            let position = reader.position();
            match event.unwrap() {
                XmlEvent::StartElement { name } => {         
                    if depth > 0 {
                        n_i += 1;                    
                        let c_i = dom.nodes[p_stack[depth - 1] as usize].first_child;
                        if c_i == NODE_MAX {
                            dom.set_child(p_stack[depth - 1], n_i);
                        }else{
                            let mut s_i = c_i; 
                            loop {
                                let ns_i = dom.nodes[s_i as usize].next_sibling;
                                if ns_i == NODE_MAX {
                                    dom.nodes[s_i as usize].next_sibling = n_i;
                                    break;
                                }else{
                                    s_i = ns_i;
                                }
                            }
                        }
                    }                                  
                    log::trace!("{position:4}| ({depth}:{n_i}) <{name}>");    

                    p_stack[depth] = n_i;
                    depth += 1;          
                }
                XmlEvent::Attribute { key, value} => {         
                    let n_i = if depth > 0 { p_stack[depth - 1] } else { 0 };                    
                    
                    log::trace!("{position:4}| ({depth}:{n_i})  {key} = \"{value}\"");

                    match key {                        
                        "vertical"=>{
                            dom.node_ref_mut(n_i).set_direction(Direction::Vertical);
                        }
                        "width"=>{
                            if value.starts_with("flex"){
                                let num = value[4..].parse::<SizeW>().unwrap_or(1);
                                dom.node_ref_mut(n_i).plot.size.width = WIDTH_FLEX + 1 - core::cmp::max(1 , core::cmp::min(num, 4));
                            }else {
                                match value {
                                    "content" => { dom.node_ref_mut(n_i).plot.size.width = WIDTH_CONTENT }                                                                   
                                    _ => {
                                        let res = value.parse::<SizeW>();
                                        if res.is_ok() {
                                            dom.node_ref_mut(n_i).set_width(res.unwrap());
                                        }else{
                                            err = Ret::InvalidAttributeValue;
                                            log::error!("illegal width \"{value}\"");
                                        }
                                    }
                                }                            
                            }
                        }
                        "height"=>{
                            if value.starts_with("flex"){
                                let num = value[4..].parse::<SizeH>().unwrap_or(1);
                                dom.node_ref_mut(n_i).plot.size.height = HEIGHT_FLEX + 1 - core::cmp::max(1 , core::cmp::min(num, 4));
                            }else {
                                match value {
                                    "content" => { dom.node_ref_mut(n_i).plot.size.height = HEIGHT_CONTENT; }                                
                                    _ => {
                                        let res = value.parse::<SizeH>();
                                        if res.is_ok() {
                                            dom.node_ref_mut(n_i).set_height(res.unwrap());
                                        }else{
                                            err = Ret::InvalidAttributeValue;
                                            log::error!("illegal height \"{value}\"");
                                        }
                                    }
                                }                            
                            }
                        }
                        "align"=>{
                            if depth > 0 {                                
                                dom.node_ref_mut(n_i).set_alignment(match value {
                                    "start"   =>{ Alignment::Start },
                                    "center"  =>{ Alignment::Center },
                                    "end"     =>{ Alignment::End },
                                    "stretch" =>{ Alignment::Stretch },
                                    _=>{
                                        err = Ret::InvalidAttributeValue;
                                        log::error!("illegal align \"{value}\"");
                                        Alignment::Start
                                    }
                                });
                            }
                        }                        
                        "hidden"=>{                            
                            dom.node_ref_mut(n_i).set_visibility(Visibility::Hidden);                            
                        }
                        "margin"=>{
                            dom.set_margin(n_i, control::decode_values(value));
                        }
                        "padding"=>{                            
                            dom.set_padding(n_i, control::decode_values(value));
                        }                        
                        "background"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );                            
                            dom.request_style(n_i).set_background(value);
                        }
                        "color"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 1; } );                                                        
                            dom.request_style(n_i).set_color(value);
                        }
                        "border" | "frame"=>{        
                            if value.eq_ignore_ascii_case("solid") {
                                dom.request_style(n_i).set_border(BORDER_RECT);                        
                            }else{
                                let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 1; } );                                                        
                                dom.request_style(n_i).set_border(value as BorderIdx);                        
                            }
                            dom.node_ref_mut(n_i).set_framing(key == "border");
                        }
                        "font"=>{                                          
                            let value: FontIdx = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );                                                  
                            dom.request_style(n_i).set_font(value);                          
                        }
                        _=>{}
                    }                     
                },               
                XmlEvent::EndElement { name } => {                         
                    depth -= 1;
                    log::trace!("{position:4}| ({depth}:{n_i}) </{name}>");
                },
                XmlEvent::CData { data } => {                        
                    let str = data;
                    if str.len() > 0 {
                        let raw = str.as_ptr();                
                        if dom.node(n_i).plot.content_idx == CONTENT_MAX && dom.node(n_i).first_child == NODE_MAX {
                        dom.set_content(n_i, ContentContainer::Raw { raw: raw, len : str.len() });
                        log::trace!("{position:4}| ({depth}:{n_i})  Content: \"{data}\"");
                        }
                    }
                },            
                XmlEvent::EndDocument {} => break,            
                _ => {}
            }
        }    
        return err;
    }
    
    #[cfg(all(feature = "dom", feature = "xml", feature = "inspect"))]
    pub fn to_xml(&mut self, xml: &mut dyn core::fmt::Write, pad: Option<&str>, incl_idx: bool) -> core::fmt::Result {                                             
        let mut p_stack: [NodeIdx; DOM_DEPTH_MAX] = [NODE_MAX; DOM_DEPTH_MAX];
        let mut n_i: NodeIdx = 0;
        let mut p_i: usize = 0;
        let mut descent: u8 = 0;

        if NODE_MAX == 0 {
            xml.write_str("").expect("");
            return Ok(());
        }        
        
        fn write_nesw(xml: &mut dyn core::fmt::Write, n: Offset, e: Offset, s: Offset, w: Offset) -> core::fmt::Result {
            if n == s {
                if e == w {
                    if n == e {
                        return xml.write_fmt(format_args!("{}", n));
                    }else{
                        return xml.write_fmt(format_args!("\"{} {}\"", n, e));
                    }
                }
            }
            xml.write_fmt(format_args!("\"{} {} {} {}\"", n, e, s, w)).expect("");
            Ok(())
        }

        loop {                    
            let node = self.nodes[n_i as usize];

            let tagname = if n_i == 0 { "body" } else { "div" };
                         
            if descent == 0 {
                if pad.is_some() { for _i in 0..p_i { xml.write_str(pad.unwrap()).expect(""); } }
                xml.write_fmt(format_args!("<{}", tagname)).expect("");                               
                
                let width = node.plot.size.width;
                if width < WIDTH_MAX {
                    xml.write_fmt(format_args!(" width={}", width)).expect("");
                }else
                if width < WIDTH_CONTENT {                    
                    xml.write_fmt(format_args!(" width=flex{}", WIDTH_FLEX - width + 1)).expect("");                            
                }
                
                let height = node.plot.size.height;
                if height < HEIGHT_MAX {
                    xml.write_fmt(format_args!(" height={}", height)).expect("");
                }else
                if height < HEIGHT_CONTENT {
                    xml.write_fmt(format_args!(" height=flex{}", HEIGHT_FLEX - height + 1)).expect("");                    
                }

                match node.visibility() {
                    Visibility::Hidden => {
                        xml.write_str(" hidden").expect("");
                    }
                    _ => {}
                }

                match node.alignment() {
                    Alignment::Center => { xml.write_str(" align=center").expect(""); },
                    Alignment::End => { xml.write_str(" align=end").expect(""); },
                    Alignment::Stretch => { xml.write_str(" align=stretch").expect(""); },
                    _ => {}
                }
                
                if node.direction() != Direction::Horizontal {
                        xml.write_str(" vertical").expect("");
                }

                let padding = self.paddings[node.plot.padding_idx as usize];            
                if padding.n != 0 || padding.e != 0 || padding.s != 0 || padding.w != 0 {
                    xml.write_str(" padding=").expect("");
                    write_nesw(xml, padding.n, padding.e, padding.s, padding.w).expect("");
                }

                let margin = self.margins[node.plot.margin_idx as usize];    
                if margin.n != 0 || margin.e != 0 || margin.s != 0 || margin.w != 0 {
                    xml.write_str(" margin=").expect("");
                    write_nesw(xml, margin.n, margin.e, margin.s, margin.w).expect("");
                }

                let style = self.styles[node.plot.style_idx as usize];
                if style.border_idx == BORDER_RECT {
                    xml.write_fmt(format_args!(" {}=solid", if node.framing() { "border" } else { "frame" })).expect("");
                }else
                if style.border_idx != BORDER_MAX {
                    xml.write_fmt(format_args!(" {}={}", if node.framing() { "border" } else { "frame" }, style.border_idx)).expect("");
                }
                if style.font_idx != FONT_MAX {
                    xml.write_fmt(format_args!(" font={}", style.font_idx)).expect("");
                }
                if style.color != 1 {
                    xml.write_fmt(format_args!(" color={}", style.color)).expect("");
                }
                if style.background != Color::MAX {
                    xml.write_fmt(format_args!(" background={}", style.background)).expect("");
                }
                if node.visibility() == Visibility::Hidden {
                    xml.write_str(" visible=0").expect("");
                }

                if incl_idx {
                    xml.write_fmt(format_args!(" _idx={}", n_i)).expect("");                    
                }
                
                xml.write_str(">").expect("");              
                if node.plot.content_idx != CONTENT_MAX && node.first_child == NODE_MAX {                    
                    let content = self.contents[node.plot.content_idx as usize];
                    if content.len > 0 {
                        if pad.is_some() { xml.write_str("\n").expect(""); }
                        if pad.is_some() { for _i in 0..p_i { xml.write_str(pad.unwrap()).expect(""); } }
                        xml.write_str(content.as_str()).expect("");              
                        if pad.is_some() { xml.write_str("\n").expect(""); }
                    }                    
                }else
                if node.first_child != NODE_MAX {
                    if pad.is_some() { xml.write_str("\n").expect(""); }
                }
            
                if node.first_child == NODE_MAX {
                    descent = 1;                        
                }else{                
                    p_stack[p_i] = n_i;
                    p_i += 1;
                    n_i = node.first_child;
                }
            }else if descent == 1 {
                if node.plot.content_idx != CONTENT_MAX || node.first_child != NODE_MAX {                    
                    if pad.is_some() { for _i in 0..p_i { xml.write_str(pad.unwrap()).expect(""); } }
                }
                xml.write_fmt(format_args!("</{}>", tagname)).expect("");                               
                if pad.is_some() { xml.write_str("\n").expect(""); }
                
                if p_i == 0 {
                    break;
                }
                
                if n_i > 0 && node.next_sibling != NODE_MAX {
                    descent = 0;
                    n_i = node.next_sibling;
                }else{
                    if p_i > 0 {
                        p_i -= 1;
                        n_i = p_stack[p_i];               
                    }else{
                        break;
                    }
                }
            }
        }     
        
        Ok(())
    }
}