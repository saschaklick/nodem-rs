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

/// Reads a build-time limit from the environment variable `value` came from, else `default`.
/// Set it in the building project's `.cargo/config.toml`, e.g. `[env] NODEM_NODE_MAX = "24"`;
/// changing it rebuilds nodem-rs. A value that is not a number in `min..=max` fails the build.
const fn limit(value: Option<&str>, default: usize, min: usize, max: usize) -> usize {
    let n = match value {
        None => default,
        Some(s) => {
            let bytes = s.as_bytes();
            if bytes.is_empty() { panic!("nodem limit: empty value"); }
            let mut n = 0usize;
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < b'0' || b > b'9' { panic!("nodem limit: not a number"); }
                n = n * 10 + (b - b'0') as usize;
                i += 1;
            }
            n
        }
    };
    if n < min || n > max { panic!("nodem limit: out of range"); }
    n
}

// Node and content indices are u8 and their *_MAX value means "none", so at most 255.
pub const NODE_MAX:      NodeIdx     = limit(option_env!("NODEM_NODE_MAX"), 64, 1, 255) as NodeIdx;
pub const CONTENT_MAX:   ContentIdx  = limit(option_env!("NODEM_CONTENT_MAX"), 64, 1, 255) as ContentIdx;
pub const AREA_MAX:      NodeIdx     = NODE_MAX;
pub const MARGIN_INIT:   MarginIdx   = 0;
pub const MARGIN_MAX:    MarginIdx   = 12;
pub const PADDING_INIT:  PaddingIdx  = 0;
pub const PADDING_MAX:   PaddingIdx  = 12;
pub const STYLE_INIT:    StyleIdx    = 0;
pub const STYLE_MAX:     StyleIdx    = limit(option_env!("NODEM_STYLE_MAX"), 32, 1, 254) as StyleIdx;
pub const BORDER_MAX:    BorderIdx   = limit(option_env!("NODEM_BORDER_MAX"), 10, 2, 16) as BorderIdx;
pub const BORDER_RECT:   BorderIdx   = BORDER_MAX - 1;
pub const FONT_MAX:      FontIdx     = FontIdx::MAX;
pub const ID_MAX:        IdIdx       = 10;

pub const DOM_DEPTH_MAX: usize       = limit(option_env!("NODEM_DOM_DEPTH_MAX"), 8, 1, 255);

/// Receive buffer for packages in builds without `alloc` (with `alloc` it is sized per package).
pub const MEDIA_SIZE:    usize       = limit(option_env!("NODEM_MEDIA_SIZE"), 1024 * 8, 16, usize::MAX);

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