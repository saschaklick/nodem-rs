#![no_std]
#![allow(static_mut_refs)]

pub use i16 as PosX;
pub use i16 as PosY;
pub use u16 as SizeW;
pub use u16 as SizeH;
pub use i8  as Offset;
pub use u8  as Color;

pub use u8      as LibraryIdx;
pub use u8      as NodeIdx;
pub use NodeIdx as AreaIdx;
pub use u8      as ContentIdx;
pub use u8      as MarginIdx;
pub use u8      as PaddingIdx;
pub use u8      as SizeIdx;
pub use u8      as PositionIdx;
pub use u8      as StyleIdx;
pub use u8      as BorderIdx;
pub use u8      as FontIdx;
pub use u8      as IdIdx;

pub const SCREEN_WIDTH:  SizeW = 128;
pub const SCREEN_HEIGHT: SizeH = 64;

pub const SCREEN_SIZE:        usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_BUFFER_SIZE: usize = SCREEN_SIZE / 8;

pub const SURFACE_WIDTH:  SizeW = SCREEN_WIDTH;
pub const SURFACE_HEIGHT: SizeH = SCREEN_HEIGHT;
pub const SURFACE_BUFFER_SIZE:usize  = (SURFACE_WIDTH as usize * SURFACE_HEIGHT as usize) / 8;

pub const LIBRARY_MAX:   LibraryIdx  = 4;

pub const NODE_MAX:      NodeIdx     = 24;
pub const CONTENT_MAX:   ContentIdx  = 16;
pub const AREA_MAX:      NodeIdx     = NODE_MAX;
pub const MARGIN_INIT:   MarginIdx   = 0;
pub const MARGIN_MAX:    MarginIdx   = 12;
pub const PADDING_INIT:  PaddingIdx  = 0;
pub const PADDING_MAX:   PaddingIdx  = 12;
pub const STYLE_INIT:    StyleIdx    = 0;
pub const STYLE_MAX:     StyleIdx    = 12;
pub const BORDER_RECT:   BorderIdx   = 10;
pub const BORDER_MAX:    BorderIdx   = BORDER_RECT + 1;
pub const FONT_MAX:      FontIdx     = FontIdx::MAX;
pub const ID_MAX:        IdIdx       = 10;

pub const DOM_DEPTH_MAX: usize       = 8;

pub mod surface;
#[cfg(feature = "dom")]
pub mod dom;
#[cfg(feature = "dom")]
pub mod plot;
#[cfg(feature = "dom")]
pub mod node;
pub mod control;
pub mod media;

#[cfg(feature = "runtime")]
pub mod runtime;

#[cfg(feature = "vm")]
pub mod int_surface;

#[cfg(all(feature = "xml"))]
pub mod xmlparser;

mod stream;

#[cfg(test)]
mod tests;

pub struct Nodem {
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: PosX,
    pub y: PosY
}
impl Default for Point {
    fn default() -> Self { Point { x: 0, y: 0 } }
}
impl Point {
    pub fn clear(&mut self) { self.x = 0; self.y = 0; }
}
impl core::ops::Add for Point {
     type Output = Point;
     fn add(self, rhs: Self) -> Self::Output {
         return Point { x: self.x.saturating_add(rhs.x), y: self.y.saturating_add(rhs.y) };
     }
}

#[derive(Copy, Clone)]
pub struct Size {
    pub width: SizeW,
    pub height: SizeH
}
impl Default for Size {
    fn default() -> Self { Size { width: 0, height: 0 } }
}
impl Size {
    fn clear(&mut self) { self.width = 0; self.height = 0; }
}
impl core::ops::Add for Size {
     type Output = Size;
     fn add(self, rhs: Self) -> Self::Output {
         return Size { width: self.width.saturating_add(rhs.width), height: self.height.saturating_add(rhs.height) };
     }
}
impl core::ops::Sub for Size {
     type Output = Size;
     fn sub(self, rhs: Self) -> Self::Output {
         return Size { width: self.width.saturating_sub(rhs.width), height: self.height.saturating_sub(rhs.height) };
     }
}

#[derive(Copy, Clone)]
pub struct Area {
    pub point: Point,
    pub size: Size
}
impl Default for Area {
    fn default() -> Self { Area { point: Point::default(), size: Size::default() } }    
}
impl Area {
    pub fn clear(&mut self) {
        self.point.x = 0;
        self.point.y = 0;
        self.size.width = 0;
        self.size.height = 0;
    }
}