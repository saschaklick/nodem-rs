pub enum ColorMode {
    Index1,
    Rgb332,
    Rgb565,
    Rgb666,
    Rgb24
}

pub struct Color {}

cfg_select! {
    nodem_mode = "rgb332" => {
        pub use u8 as ColorType;        
        impl Color {
            pub const MODE: ColorMode = ColorMode::Rgb332;
            pub const BITS_PER_PIXEL: usize = 8;
            pub const COLORS: [ColorType;2] = [0b00_000_000, 0b11_111_111];        
            pub const TRANSPARENT: ColorType = 0b00_000_001;        
            pub fn invert(color: ColorType) -> ColorType { return color ^ 0xff; }            
            pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorType { return (r >> 5 << 5) | (g >> 5 << 2) | (b >> 6); }
            pub fn to_rgb(color: ColorType) -> [u8;3] { return [(color & 0b00_000_111) << 5, (color & 0b00_111_000) << 3, color & 0b11_00_000]; }
        }        
    }
    nodem_mode = "rgb565" => {
        pub use u16 as ColorType;        
        impl Color {
            pub const MODE: ColorMode = ColorMode::Rgb565;
            pub const BITS_PER_PIXEL: usize = 16;
            pub const COLORS: [ColorType;2] = [0b00000_000000_00000, 0b11111_111111_11111];        
            pub const TRANSPARENT: ColorType = 0b00000_000000_00001;        
            pub fn invert(color: ColorType) -> ColorType { return color ^ 0xffff; }            
            pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorType { return (b as u16 >> 3) | ((g as u16 >> 2) << 5) | ((r as u16 >> 3) << 11); }
            pub fn to_rgb(color: ColorType) -> [u8;3] { return [((color & 0b00000_000000_11111) << 3) as u8, ((color & 0b00000_111111_00000) >> 3) as u8, ((color & 0b11111_000000_00000) >> 8) as u8]; }            
        }                
    }
    nodem_mode = "rgb666" => {        
        pub use u32 as ColorType;        
        impl Color {
            pub const MODE: ColorMode = ColorMode::Rgb666;
            pub const BITS_PER_PIXEL: usize = 24;
            pub const COLORS: [ColorType;2] = [0b00000000_00000000_00000000, 0b00111111_00111111_00111111];        
            pub const TRANSPARENT: ColorType = 0b10000000_00000000_00000000;        
            pub fn invert(color: ColorType) -> ColorType { return color ^ 0b00111111_00111111_00111111; }            
            pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorType { return (r as u32) >> 2 | (g as u32 >> 2) << 8 | (b as u32 >> 2) << 16 }
            pub fn to_rgb(color: ColorType) -> [u8;3] { return [(color >> 2) as u8 & 0x3f, (color >> 8) as u8 & 0x3f, (color >> 16) as u8 & 0x3f]; }
        }
    }
    nodem_mode = "rgb24" => {        
        pub use u32 as ColorType;        
        impl Color {
            pub const MODE: ColorMode = ColorMode::Rgb24;
            pub const BITS_PER_PIXEL: usize = 24;
            pub const COLORS: [ColorType;2] = [0b00000000_00000000_00000000, 0b11111111_11111111_11111111];        
            pub const TRANSPARENT: ColorType = 0b00000000_00000000_00000001;        
            pub fn invert(color: ColorType) -> ColorType { return color ^ 0xffffff; }            
            pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorType { return (r as u32) << 0 | (g as u32) << 8 | (b as u32) << 16 }
            pub fn to_rgb(color: ColorType) -> [u8;3] { return [(color >> 0) as u8, (color >> 8) as u8, (color >> 16) as u8]; }
        }
    }
    _ => {
        pub use u8 as ColorType;        
        impl Color {
            pub const MODE: ColorMode = ColorMode::Index1;
            pub const BITS_PER_PIXEL: usize = 1;
            pub const COLORS: [ColorType;2] = [0b0, 0b1];
            pub const TRANSPARENT: ColorType = 0b11111111; 
            pub fn invert(color: ColorType) -> ColorType { return if color > 0 { 0b0 } else { 0b1 }; }            
            pub fn from_rgb(r: u8, g: u8, b: u8) -> ColorType { return if ((r >> 2) + (g >> 2) + (b >> 2)) > 16 { 0b1 } else { 0b0 }; }
            pub fn to_rgb(color: ColorType) -> [u8;3] { return if color == 0 { [ 0x00, 0x00, 0x00 ] } else { [ 0xff, 0xff, 0xff ] }; }
        }        
    }
}

impl Color {
    pub fn parse(str: &str) -> Result<ColorType, &str> {
        Ok(match str {
            ""|"transparent"  => Color::TRANSPARENT,
            "0" => Color::COLORS[0],
            "1" => Color::COLORS[1],            
            _ => {                                
                if str.starts_with("#") {
                    if str.len() == 4 {
                        let nibs = [u8::from_str_radix(&str[1..4], 16).unwrap_or(0), u8::from_str_radix(&str[2..3], 16).unwrap_or(0), u8::from_str_radix(&str[3..4], 16).unwrap_or(0)];
                        Color::from_rgb(nibs[0] << 4 | nibs[0], nibs[1] << 4 | nibs[1], nibs[2] << 4 | nibs[2])
                    }else
                    if str.len() == 7 {
                        let bytes = [u8::from_str_radix(&str[1..3], 16).unwrap_or(0), u8::from_str_radix(&str[3..5], 16).unwrap_or(0), u8::from_str_radix(&str[5..7], 16).unwrap_or(0)];
                        Color::from_rgb(bytes[0], bytes[1], bytes[2])
                    }else{
                        return Err("malformed color definiton");
                    }

                }else{
                    return Err("malformed color definiton");
                }
            }            
        })
    }
}
