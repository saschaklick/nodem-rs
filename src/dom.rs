use core::fmt;
#[cfg(feature = "alloc")]
use nostd::alloc::{ alloc, Layout };
use crate::*;
use crate::node::*;
use crate::node::id::*;
use crate::node::style::*;
use crate::plot::content::Content;

mod calculate;
mod page;
mod surface;

#[derive(Copy, Clone)]
pub struct Margin {
    pub n: Offset,
    pub e: Offset,
    pub s: Offset,
    pub w: Offset
}
impl Default for Margin {
    fn default() -> Self { Margin { n: 0, e: 0, s: 0, w: 0 } }
}

impl Margin {
    pub fn reset(&mut self) {
        self.n = 0;
        self.e = 0;
        self.s = 0;
        self.w = 0;
    }
}

#[derive(Copy, Clone)]
pub struct Padding {
    pub n: Offset,
    pub e: Offset,
    pub s: Offset,
    pub w: Offset
}
impl Default for Padding {
    fn default() -> Self { Padding { n: 0, e: 0, s: 0, w: 0 } }
}

impl Padding {
    pub fn reset(&mut self) {
        self.n = 0;
        self.e = 0;
        self.s = 0;
        self.w = 0;
    }
}

pub enum ContentContainer<'a> {
    Raw { raw: *const u8, len: usize },
    Str(&'a str)
}
struct DOMContainer {
    raw: *mut u8,
    len: usize    
}

#[repr(u8)]
pub enum Ret {
    Ok = 0,
    InvalidXML = 1,
    InvalidBin = 2,
    InvalidFormat = 3,
    InvalidSource = 4,
    InvalidProperty = 5,
    InvalidArgument = 6,
    InvalidDOMInstruction = 7,
    InvalidAttributeValue = 8,    
    #[cfg(not(feature = "alloc"))]
    NoAllocFeature = 12,
    #[cfg(feature = "alloc")]
    AllocFailure = 13
}

pub struct DOM {
    pub nodes: [Node; NODE_MAX as usize],
    #[cfg(feature = "inspect")]
    pub positions: NodePosition,
    pub contents: [Content; CONTENT_MAX as usize],
    pub content_idx: ContentIdx,
    pub ids: [Id; ID_MAX as usize],
    pub id_idx: IdIdx,
    pub paddings: [Padding; PADDING_MAX as usize],
    pub padding_idx: PaddingIdx,
    pub margins: [Margin; PADDING_MAX as usize],
    pub margin_idx: MarginIdx,
    pub styles: [Style; STYLE_MAX as usize + 1],
    pub style_idx: StyleIdx,
    
    dummy_node: Node,
    #[cfg(feature = "alloc")]
    alloc_buf:  DOMContainer
}
impl Default for DOM {
    fn default() -> Self { DOM {
        nodes: [Node::default(); NODE_MAX as usize],
        #[cfg(feature = "inspect")]
        positions: NodePosition::new(),
        contents: [Content::default(); CONTENT_MAX as usize],           
        content_idx: 0,
        ids: [Id::default(); ID_MAX as usize],           
        id_idx: 0,
        paddings: [Padding::default(); PADDING_MAX as usize],           
        padding_idx: PADDING_INIT + 1,
        margins: [Margin::default(); MARGIN_MAX as usize],           
        margin_idx: MARGIN_INIT + 1,
        styles: [Style::default(); STYLE_MAX as usize + 1],           
        style_idx: STYLE_INIT + 1,        

        dummy_node: Node::default(),        
        #[cfg(feature = "alloc")]
        alloc_buf: DOMContainer { raw: core::ptr::null_mut(), len: 0 }
    } }
}

impl DOM {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn reset(&mut self) {        
        self.content_idx = 0;        
        self.padding_idx = PADDING_INIT + 1;        
        self.margin_idx = MARGIN_INIT + 1;        
        self.style_idx = STYLE_INIT + 1;
        for i in 0 as usize .. NODE_MAX as usize{ self.nodes[i as usize].reset(); }
        for i in 0 as usize .. MARGIN_MAX as usize{ self.margins[i as usize].reset(); }
        for i in 0 as usize .. PADDING_MAX as usize{ self.paddings[i as usize].reset(); }
        for i in 0 as usize .. STYLE_MAX as usize { self.styles[i as usize].reset(); }
        for i in 0 as usize .. CONTENT_MAX as usize { self.contents[i as usize].reset(); }

        self.nodes[0].plot.size = Size { width: WIDTH_FLEX, height: HEIGHT_FLEX };
    }

    pub fn clear(&mut self) {
        self.reset();
        for node in &mut self.nodes {
            node.reset();
        }
    }

    pub fn node(&self, node_idx: NodeIdx ) -> Node {
        if node_idx < NODE_MAX {
            return self.nodes[node_idx as usize];
        }else{
            return self.dummy_node;
        }
    }

    pub fn node_ref(&self, node_idx: NodeIdx ) -> &Node {
        if node_idx < NODE_MAX {
            return &self.nodes[node_idx as usize];
        }else{
            return &self.dummy_node;
        }
    }

    pub fn node_ref_mut(&mut self, node_idx: NodeIdx ) -> &mut Node {
        if node_idx < NODE_MAX {
            return &mut self.nodes[node_idx as usize];
        }else{
            return &mut self.dummy_node;
        }
    }

    pub fn set_child(&mut self, parent_idx: NodeIdx, child_idx: NodeIdx) {
        if parent_idx < NODE_MAX && child_idx <= NODE_MAX {
            self.nodes[parent_idx as usize].first_child = child_idx;
        }
    }

    pub fn remove_child(&mut self, parent_idx: NodeIdx) {
        if parent_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            self.nodes[parent_idx as usize].first_child = NODE_MAX;
        }
    }

    pub fn set_sibling(&mut self, first_idx: NodeIdx, sibling_idx: NodeIdx) {
        if first_idx > NODE_MAX || sibling_idx > NODE_MAX {
            log::error!("illegal node index");
        }else{
            self.nodes[first_idx as usize].next_sibling = sibling_idx;
        }
    }

    pub fn remove_sibling(&mut self, first_idx: NodeIdx) {
        if first_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            self.nodes[first_idx as usize].next_sibling = NODE_MAX;
        }
    }

    pub fn remove(&mut self, node_idx: NodeIdx) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{            
            let node = self.nodes[node_idx as usize];
            for n in &mut self.nodes {                
                if n.first_child == node_idx { n.first_child = node.next_sibling; }
                if n.next_sibling == node_idx { n.next_sibling = node.next_sibling; }                
            }
            self.nodes[node_idx as usize].reset();
        }
    }

    pub fn set_padding(&mut self, node_idx: NodeIdx, values: [Offset;4]) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            let node = &mut self.nodes[node_idx as usize];
            let mut p_i = node.plot.padding_idx;
            if p_i == 0 {
                if self.padding_idx >= PADDING_MAX {
                    log::error!("too many paddings");
                    return;
                }
                p_i = self.padding_idx;
                self.padding_idx += 1;
                node.plot.padding_idx = p_i;
            }
            let padding = &mut self.paddings[p_i as usize];                 
            padding.n = values[0];
            padding.e = values[1];
            padding.s = values[2];
            padding.w = values[3];
        }
    }

    pub fn set_margin(&mut self, node_idx: NodeIdx, values: [Offset;4]) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            let node = &mut self.nodes[node_idx as usize];
            let mut p_i = node.plot.margin_idx;
            if p_i == 0 {
                p_i = self.margin_idx;
                self.margin_idx += 1;
                node.plot.margin_idx = p_i;
            }
            let margin = &mut self.margins[p_i as usize];            
            margin.n = values[0];
            margin.e = values[1];
            margin.s = values[2];
            margin.w = values[3];
        }
    }

    pub fn set_border(&mut self, node_idx: NodeIdx, border_idx: BorderIdx) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            let node = &mut self.nodes[node_idx as usize];            
            if node.plot.style_idx == STYLE_INIT {
                if self.style_idx >= STYLE_MAX {
                    log::error!("too many styles");
                    return;
                }                
                self.style_idx += 1;                
                node.plot.style_idx = self.style_idx;
            }
            let style = &mut self.styles[node.plot.style_idx as usize];
            style.border_idx = border_idx;
        }
    }

    pub fn set_content(&mut self, node_idx: NodeIdx, value: ContentContainer) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");                
        }else{
            let node = &mut self.nodes[node_idx as usize];
            let mut c_i = node.plot.content_idx;
            if c_i == CONTENT_MAX {
                c_i = self.content_idx;
                self.content_idx += 1;
                node.plot.content_idx = c_i;
            }
            let content= &mut self.contents[c_i as usize];            
            match value {                                
                ContentContainer::Raw { raw, len } => {                                
                    content.raw = raw;            
                    content.len = len;                    
                }                
                #[cfg(feature = "alloc")]
                ContentContainer::Str(value) => {                                                            
                    unsafe {                        
                        if content.alloc {
                            use nostd::alloc::dealloc;
                            dealloc(content.raw as *mut u8, Layout::from_size_align(content.len, 4).unwrap());
                        }
                        let buf = alloc(Layout::from_size_align(value.len(), 4).unwrap());
                        if buf != core::ptr::null_mut() {
                            buf.copy_from(value.as_ptr(), value.len());
                            content.raw = buf;
                            content.len = value.len();
                            content.alloc = true;
                        }else{
                            log::error!("alloc failed");
                        }                    
                    }
                }
                #[cfg(not(feature = "alloc"))]
                ContentContainer::Str(_) => {                                                            
                    log::error!("no alloc feature; cannot set content");
                    return;
                }
            }
            node.first_child = NODE_MAX;
        }   
    }

    pub fn set_id(&mut self, node_idx: NodeIdx, raw: *const u8, len: usize) {
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
        }else{
            let node = &mut self.nodes[node_idx as usize];
            let mut i_i = node.id_idx;
            if i_i == ID_MAX {
                i_i = self.id_idx;
                self.id_idx += 1;
                node.id_idx = i_i;
            }
            let id= &mut self.ids[i_i as usize];            
            id.raw = raw;            
            id.len = len;
        }   
    }

    pub fn request_style<'a>(&mut self, node_idx: NodeIdx) -> &mut Style {        
        if node_idx >= NODE_MAX {
            log::error!("illegal node index");
            return &mut self.styles[STYLE_MAX as usize];
        }else{
            let node = &mut self.nodes[node_idx as usize];
            if node.plot.style_idx == STYLE_INIT {
                if self.style_idx >= STYLE_MAX {
                    log::error!("too many styles");
                    return &mut self.styles[STYLE_MAX as usize];
                }
                node.plot.style_idx = self.style_idx;
                self.style_idx += 1;
            }
            return &mut self.styles[node.plot.style_idx as usize];
        }
    }
}

impl fmt::Display for DOM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        write!(f, "P: ({}/{}) M: ({}/{}) C: ({}/{}) S: ({}/{})", self.padding_idx, PADDING_MAX, self.margin_idx, MARGIN_MAX, self.content_idx, CONTENT_MAX, self.style_idx, STYLE_MAX)?;   
        return write!(f, "");
    }
}


impl DOM {    
    #[cfg(feature = "inspect")]                             
    pub fn inspect(&mut self, xml: &mut dyn core::fmt::Write) -> core::fmt::Result {     
        xml.write_str("idx,first_child,next_sibling,alignment,visible,direction,framing,width,height,p_n,p_e,p_s,p_w,m_n,m_e,m_s,m_w,color,background,border_idx,font_idx,content").expect("");        
        xml.write_str(",_left,_top,_width,_height,_cleft,_ctop,_cwidth,_cheight").expect("");        
        xml.write_str("\r\n").expect("");        
        xml.write_fmt(format_args!(                
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"\",{},{},{},{}\r\n",
                NODE_MAX, NODE_MAX, NODE_MAX,
                4, 2, 2, 2, WIDTH_CONTENT, HEIGHT_CONTENT,
                Offset::MAX, Offset::MAX, Offset::MAX, Offset::MAX,
                Offset::MAX, Offset::MAX, Offset::MAX, Offset::MAX,
                Color::MAX, Color::MAX, BORDER_MAX, FONT_MAX,
                PosX::MAX, PosY::MAX, SizeW::MAX, SizeH::MAX
        )).expect("");
        for n_i in 0..NODE_MAX as usize {
            let node = self.nodes[n_i];
            let padding = self.paddings[node.plot.padding_idx as usize];
            let margin = self.margins[node.plot.margin_idx as usize];
            let style = self.styles[node.plot.style_idx as usize];
            let content = if node.plot.content_idx < CONTENT_MAX { self.contents[node.plot.content_idx as usize].as_str() } else { "" };
            xml.write_fmt(format_args!(                
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\"",
                n_i,
                node.first_child,
                node.next_sibling,
                node.alignment() as u8,
                node.visibility() as u8,
                node.direction() as u8,
                node.framing() as u8,
                node.plot.size.width,
                node.plot.size.height,                
                padding.n,
                padding.e,
                padding.s,
                padding.w,
                margin.n,
                margin.e,
                margin.s,
                margin.w,
                style.color,               
                style.background,
                style.border_idx,
                style.font_idx,
                content
            )).expect("");
            let position = self.positions.areas[n_i];
            xml.write_fmt(format_args!(                
                ",{},{},{},{},{},{},{},{}",
                position.0.point.x,
                position.0.point.y,
                position.0.size.width,
                position.0.size.height,
                position.0.point.x.saturating_add(margin.w.saturating_add(padding.w) as i16),
                position.0.point.y.saturating_add(margin.n.saturating_add(padding.n) as i16),
                core::cmp::max(0, (position.0.size.width as i16).saturating_sub(padding.e.saturating_add(padding.w).saturating_add(margin.e).saturating_add(margin.w) as i16)),
                core::cmp::max(0, (position.0.size.height as i16).saturating_sub(padding.n.saturating_add(padding.s).saturating_add(margin.n).saturating_add(margin.s) as i16))
            )).expect("");
            xml.write_str("\r\n").expect("");        
        }
        Ok(())
    }
}