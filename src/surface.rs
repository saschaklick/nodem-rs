use core::cmp;
use crate::*;
use crate::media::*;

pub mod image;
pub mod font;
pub mod ninepatch;

pub const IMAGE:       char = 0x01 as char;
pub const FONT:        char = 0x02 as char;
pub const DRAWING:     char = 0x03 as char;
pub const TYPESETTING: char = 0x04 as char;

#[derive(Copy, Clone)]
pub struct Clip {
    pub p0: Point,
    pub p1: Point
}
impl Default for Clip {
    fn default() -> Self {
        Clip { p0: Point { x: 0, y: 0 }, p1 : Point { x : PosX::MAX, y: PosY::MAX } }
    }   
}
impl Clip {    
    pub fn reset(&mut self, width: SizeW, height: SizeH) {
        self.p0.x = 0;
        self.p0.y = 0;
        self.p1.x = width as PosX;
        self.p1.y = height as PosY;
    }
}

pub struct Surface {
    pub framebuffer: *mut [u8],
    pub width: SizeW,
    pub height: SizeH,
    pub palette: [Color;2],
    pub clip: Clip,
    pub media: Media    
}
impl Surface {
    pub fn new(framebuffer: *mut [u8], width: SizeW, height: SizeH) -> Self {        
        let ret = Self { framebuffer: framebuffer, width: width, height: height, palette: [0, 1], clip: Clip { p0: Point { x: 0, y: 0 }, p1: Point { x: width as PosX, y: height as PosY } }, media: Media::default() };        
        return ret;
    }

    pub fn resize(self: &mut Self, framebuffer: *mut [u8], width: SizeW, height: SizeH){
        self.framebuffer = framebuffer;
        self.width = width;
        self.height = height;
        self.clip.reset(self.width, self.height);    
    }

    pub fn clear(self: &Self, color: Color) {        
        self.fill_rect(Area { point: Point { x: 0, y: 0 }, size: Size { width: self.width, height: self.height } }, color);
    }

    pub fn get_color(&self, index: usize) -> Color {
        return self.palette[cmp::min(self.palette.len() - 1, index)];
    }

    pub fn set_clip(&mut self, p_0: Point, p_1: Point) {
        self.clip.p0 = p_0;
        self.clip.p1 = p_1;
    }

    fn draw_pixel_unsafe(&self, p: Point, color: Color) {            
        let pixel_i = p.y as usize * self.width as usize + p.x as usize;
        unsafe {
            let framebuffer = self.framebuffer.as_mut().unwrap();
            if pixel_i / 8 < framebuffer.len() {
                if color == 0 {        
                    framebuffer[pixel_i / 8] &= !(1 << (7 - (pixel_i % 8)));
                }else{
                    framebuffer[pixel_i / 8] |= 1 << (7 - (pixel_i % 8));
                }
            }
        }
    }
    
    pub fn draw_pixel(&self, p: Point, color: Color) {                                        
        if p.x < self.clip.p0.x || p.y < self.clip.p0.y || p.x >= self.clip.p1.x || p.y >= self.clip.p1.y {
            return;
        }
        
        self.draw_pixel_unsafe(p, color);
    }
    
    pub fn draw_hline(&self, p: Point, length: SizeW, color: Color) {    
        if length <= 0 || p.y < 0 || p.y >= self.height as PosY {
            return;
        }
        let mut p_d = Point { x: 0, y: p.y };
        for x_i in cmp::max(p.x, self.clip.p0.x) .. cmp::min(p.x + length as PosX, self.clip.p1.x) {
            p_d.x = x_i;
            self.draw_pixel(p_d, color);
        }
    }
    
    pub fn draw_vline(&self, p: Point, length: SizeH, color: Color) {    
        if length <= 0 || p.y < 0 || p.y >= self.height as PosY {
            return;
        }
        let mut p_d = Point { x: p.x, y: 0 };
        for y_i in cmp::max(p.y, self.clip.p0.y) .. cmp::min(p.y + length as PosY, self.clip.p1.y) {
            p_d.y = y_i;
            self.draw_pixel(p_d, color);
        }
    }
    
    pub fn draw_rect(&self, a: Area, color: Color) {
        if a.size.width >= 2 {
            self.draw_hline(a.point, a.size.width as SizeW, color);
            if a.size.height > 1 && a.size.height as PosY - 1 < self.height as PosY - a.point.y {
                self.draw_hline(Point { x: a.point.x, y: a.point.y + a.size.height as PosY - 1 }, a.size.width as SizeW, color);
            }
        }
        if a.size.height >= 2 {
            self.draw_vline(a.point, a.size.height as SizeH - 1, color);
            if a.size.width > 1 && a.size.width as PosX - 1 < self.width as PosX - a.point.x {
                self.draw_vline(Point { x: a.point.x + a.size.width as PosX - 1, y: a.point.y + 1 }, a.size.height - 2, color);
            }
        }
    }
    
    pub fn fill_rect(&self, a: Area, color: Color) {
        if a.size.width >= 1 && a.size.height > 0 {
            for y in cmp::max(a.point.y, 0) .. cmp::min(a.point.y + a.size.height as PosY, self.height as PosY) {                
                self.draw_hline(Point { x: a.point.x, y }, a.size.width, color);
            }
        }
    }

    pub fn draw_line(&self, p_0: Point, p_1: Point, color: Color) {
        let dx = (p_1.x - p_0.x).abs();
        let sx = if p_0.x < p_1.x { 1 } else { -1 };
        let dy = -(p_1.y - p_0.y).abs();
        let sy = if p_0.y < p_1.y { 1 } else { -1 };
        let mut error = dx + dy;
        let mut p = p_0.clone();
    
        loop {
            self.draw_pixel(p, color);            
            let e2 = 2 * error;
            if e2 >= dy {
                if p.x == p_1.x { break; }
                error = error + dy;
                p.x = p.x + sx;
            }
            if e2 <= dx {
                if p.y == p_1.y { break; }
                error = error + dx;
                p.y = p.y + sy;
            }
        }
    }

    pub fn draw_progress_screen(&self, progress:u8, state: u8) {
        self.clear(0);
            
        if progress < u8::MAX {
            self.draw_rect(Area { point: Point { x: 8,  y: ((self.height / 2).saturating_sub(3)) as PosY }, size: Size { width: self.width.saturating_sub(16), height: 5 } }, 1);
            self.fill_rect(Area { point: Point { x: 10, y: ((self.height / 2).saturating_sub(1)) as PosY }, size: Size { width: self.width.saturating_sub(18) as SizeW * progress as SizeW / u8::MAX as SizeW, height: 1 } }, 1);                    
        }

        for i in 0 .. state as PosX * 2 {
            self.draw_pixel(Point { x: 2 + i, y: 2 }, 1);
        }
    }    
}