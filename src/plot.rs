use crate::node::{HEIGHT_FLEX, WIDTH_FLEX};
use crate::*;
use crate::surface::*;

pub mod content;

pub const WIDTH_CONTENT        :SizeW = SizeW::MAX;
pub const HEIGHT_CONTENT       :SizeH = SizeH::MAX;

#[derive(Copy, Clone)]
pub struct Plot {
    pub content_idx:  ContentIdx,
    pub margin_idx:   MarginIdx,
    pub padding_idx:  PaddingIdx,
    pub style_idx:    StyleIdx,
    pub size:         Size
}

const DEFAULT_SIZING: usize = 0;

impl Plot {
    pub fn reset(&mut self) {
        self.margin_idx =   MARGIN_INIT;
        self.content_idx =  CONTENT_MAX;
        self.padding_idx =  PADDING_INIT;
        self.style_idx =    STYLE_INIT;        
        self.size.width =   [WIDTH_CONTENT, WIDTH_FLEX][DEFAULT_SIZING];
        self.size.height =  [HEIGHT_CONTENT, HEIGHT_FLEX][DEFAULT_SIZING];
    }
}
impl Default for Plot {
    fn default() -> Self { Plot {
        content_idx:  CONTENT_MAX,
        margin_idx:   MARGIN_INIT,
        padding_idx:  PADDING_INIT,   
        style_idx:    STYLE_INIT,   
        size:         Size {
            width:        [WIDTH_CONTENT, WIDTH_FLEX][DEFAULT_SIZING], 
            height:       [HEIGHT_CONTENT, HEIGHT_FLEX][DEFAULT_SIZING]
        }
    } }
}


impl Surface {    
}