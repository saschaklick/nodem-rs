use crate::*;

#[derive(Copy, Clone)]
pub struct Style {
    pub color: ColorType,
    pub background: ColorType,    
    pub border_idx: BorderIdx,
    pub font_idx: FontIdx
}
impl Default for Style {
    fn default() -> Self { Style { 
        color: Color::COLORS[1],
        background: Color::TRANSPARENT,        
        border_idx: BORDER_MAX,
        font_idx:   FONT_MAX
    } }
}

impl Style {
    pub fn reset(&mut self) {
        self.color = Color::COLORS[1];
        self.background = Color::TRANSPARENT;
        self.border_idx = BORDER_MAX;
        self.font_idx = FONT_MAX; 
    }

    pub fn set_background(&mut self, color: ColorType) {                
        self.background = color;
    }

    pub fn set_color(&mut self, color: ColorType) {                
        self.color = color;
    }

    pub fn set_border(&mut self, border: BorderIdx) {                
        if border > BORDER_MAX {
            log::error!("invalid border");
            self.border_idx = BORDER_MAX;
        }else{
            self.border_idx = if border <= BORDER_MAX { border } else { BORDER_MAX };
        }
    }

    pub fn set_font(&mut self, font: FontIdx) {                
        if font > FONT_MAX {
            log::error!("invalid font");
        }else{
            self.font_idx = font;
        }
    }
}