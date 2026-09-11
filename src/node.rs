use core::fmt;
use crate::*;
use crate::plot::*;

pub mod id;
pub mod style;

#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Render { None, Flex }

#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Visibility { Hidden, Visible }

#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Direction { Horizontal, Vertical }

#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Alignment { Start, Center, End, Stretch }

pub const FLAGS_RENDER_SHIFT     :u8 = 0;
pub const FLAGS_RENDER_MASK      :u8 = 0b1;
pub const FLAGS_VISIBILITY_SHIFT :u8 = 1;
pub const FLAGS_VISIBILITY_MASK  :u8 = 0b1;
pub const FLAGS_DIRECTION_SHIFT  :u8 = 2;
pub const FLAGS_DIRECTION_MASK   :u8 = 0b1;
pub const FLAGS_OVERFLOW_SHIFT   :u8 = 3;
pub const FLAGS_OVERFLOW_MASK    :u8 = 0b1;
pub const FLAGS_ALIGNMENT_SHIFT  :u8 = 4;
pub const FLAGS_ALIGNMENT_MASK   :u8 = 0b11;
pub const FLAGS_FRAMING_SHIFT    :u8 = 6;
pub const FLAGS_FRAMING_MASK     :u8 = 0b1;

pub const FLEX_MAX: u8 = 4;

pub const WIDTH_CONTENT        :SizeW = SizeW::MAX;
pub const WIDTH_FLEX           :SizeW = WIDTH_CONTENT - 1;
pub const WIDTH_MAX            :SizeW = WIDTH_FLEX - FLEX_MAX as SizeW;

pub const HEIGHT_CONTENT       :SizeH = SizeH::MAX;
pub const HEIGHT_FLEX          :SizeH = HEIGHT_CONTENT - 1;
pub const HEIGHT_MAX           :SizeH = HEIGHT_FLEX - FLEX_MAX as SizeH;

#[derive(Copy, Clone)]
pub struct Node {       
    pub first_child:  NodeIdx,
    pub next_sibling: NodeIdx,
    pub id_idx:       IdIdx,
    pub flags:        u8,
    pub plot:         Plot
}

impl Default for Node {
    fn default() -> Self { Node {        
        first_child:  NODE_MAX,
        next_sibling: NODE_MAX,
        id_idx:       ID_MAX,
        flags: (0b1 << FLAGS_RENDER_SHIFT) | (0b1 << FLAGS_VISIBILITY_SHIFT) | (0b0 << FLAGS_DIRECTION_SHIFT) | (0b0 << FLAGS_OVERFLOW_SHIFT) | (0b1 << FLAGS_FRAMING_SHIFT),
        plot : Plot::default()
    } }
}

impl Node {
    pub fn reset(&mut self) {                
        self.first_child =  NODE_MAX;
        self.next_sibling = NODE_MAX;
        self.id_idx =       ID_MAX;
        self.flags =        (0b1 << FLAGS_RENDER_SHIFT) | (0b1 << FLAGS_VISIBILITY_SHIFT) | (0b0 << FLAGS_DIRECTION_SHIFT) | (0b0 << FLAGS_OVERFLOW_SHIFT) | (0b1 << FLAGS_FRAMING_SHIFT);
        self.plot.reset();
    }    

    pub fn set_render(&mut self, render: Render) {
        self.flags = (self.flags & !(FLAGS_RENDER_MASK << FLAGS_RENDER_SHIFT)) | ((render as u8 & FLAGS_RENDER_MASK) << FLAGS_RENDER_SHIFT);                                
    }
    pub fn render(&self) -> Render {
        return if (self.flags >> FLAGS_RENDER_SHIFT) & (FLAGS_RENDER_MASK) == FLAGS_RENDER_MASK { Render::Flex } else { Render::None }
    }

    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.flags = (self.flags & !(FLAGS_VISIBILITY_MASK << FLAGS_VISIBILITY_SHIFT)) | ((visibility as u8 & FLAGS_VISIBILITY_MASK) << FLAGS_VISIBILITY_SHIFT);                                
    }
    pub fn visibility(&self) -> Visibility {
        return if (self.flags >> FLAGS_VISIBILITY_SHIFT) & (FLAGS_VISIBILITY_MASK) == FLAGS_VISIBILITY_MASK { Visibility::Visible } else { Visibility::Hidden }
    }

    pub fn set_direction(&mut self, direction: Direction) {                         
        self.flags = (self.flags & !(FLAGS_DIRECTION_MASK << FLAGS_DIRECTION_SHIFT)) | ((direction as u8 & FLAGS_DIRECTION_MASK) << FLAGS_DIRECTION_SHIFT);                                        
    }
    pub fn direction(&self) -> Direction {           
        return if ((self.flags >> FLAGS_DIRECTION_SHIFT) & FLAGS_DIRECTION_MASK) == FLAGS_DIRECTION_MASK { Direction::Vertical } else { Direction::Horizontal }
    }

    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.flags = (self.flags & !(FLAGS_ALIGNMENT_MASK << FLAGS_ALIGNMENT_SHIFT)) | ((alignment as u8 & FLAGS_ALIGNMENT_MASK) << FLAGS_ALIGNMENT_SHIFT);                                
    }
    pub fn alignment(&self) -> Alignment {
        let value = (self.flags >> FLAGS_ALIGNMENT_SHIFT) & (FLAGS_ALIGNMENT_MASK);
        match value {
            0 => { return Alignment::Start }
            1 => { return Alignment::Center }
            2 => { return Alignment::End }
            3 => { return Alignment::Stretch }
            _ => { return Alignment::Start }
        }        
    }

    pub fn set_framing(&mut self, state: bool) {                         
        self.flags = (self.flags & !(FLAGS_FRAMING_MASK << FLAGS_FRAMING_SHIFT)) | ((state as u8 & FLAGS_FRAMING_MASK) << FLAGS_FRAMING_SHIFT);                                        
    }
    pub fn framing(&self) -> bool {           
        if ((self.flags >> FLAGS_FRAMING_SHIFT) & FLAGS_FRAMING_MASK) == FLAGS_FRAMING_MASK { true } else { false }
    }

    pub fn set_size(&mut self, size: Size) {
        self.set_width(size.width);
        self.set_height(size.height);        
    }

    pub fn set_width(&mut self, width: SizeW) {
        self.plot.size.width = width;            
    }

    pub fn set_height(&mut self, height: SizeH) {
        self.plot.size.height = height;            
    }

    pub fn set_flex(&mut self, flex_w: u8, flex_h: u8) {            
        if flex_w == 0 {
            self.plot.size.width = WIDTH_CONTENT;
        }else
        if flex_w > 0 && flex_w < FLEX_MAX {
            self.plot.size.width = WIDTH_FLEX - (flex_w - 1) as SizeW;
        }
        if flex_h == 0 {
            self.plot.size.height = HEIGHT_CONTENT;
        }else
        if flex_h > 0 && flex_h < FLEX_MAX {
            self.plot.size.height = HEIGHT_FLEX - (flex_h - 1) as SizeH;
        }
    }
}

pub struct NodePosition {
    pub areas: [(Area, Size); AREA_MAX as usize]   
}
impl NodePosition {
    pub fn new() -> Self {
        return Self { areas : [(Area { point: Point { x: 0, y: 0 }, size: Size { width: 0, height: 0 } }, Size { width: 0, height: 0 }); AREA_MAX as usize] };
    }

    pub fn clear(positions: &mut NodePosition) {        
        for c_i in 0 .. AREA_MAX as usize {
            positions.areas[c_i].0.clear();            
            positions.areas[c_i].1.clear();            
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        if self.first_child != NODE_MAX { write!(f, "C:#{:02} ", self.first_child)?; } else { write!(f, "C: -  ")?; }
        if self.next_sibling != NODE_MAX { write!(f, "S:#{:02} ", self.next_sibling)?; } else { write!(f, "S: -  ")?; }        
        write!(f, "D:{:1} ", self.direction() as u8)?;                
        write!(f, "R:{:1} ", self.render() as u8)?;                
        write!(f, "V:{:1} ", self.visibility() as u8)?;                        
        write!(f, "A:{:1}", self.alignment() as u8)?;              
        write!(f, "F:{:1}", self.framing() as u8)?;    
        write!(f, " | ")?;
        return write!(f, "[{:5}, {:3}]", self.plot.size.width, self.plot.size.height);
    }
}