#[cfg(test)]
extern crate std;

use crate::{Size, surface::Surface};
#[cfg(feature = "dom")]
use crate::{ dom:: DOM };

use bitmap_writer::{Style, Frame, Bitmap, Writer};

pub static PKG_SYS: &'static [u8] = include_bytes!("sys.pkg");

pub struct SurfaceTest {}
impl SurfaceTest {
    pub fn run<F>(size: Size, mut func:F, expected: &str) -> bool where F: FnMut(&mut Surface) {
        let mut generated = std::vec![0u8;(size.width as usize * size.height as usize + 7) / 8];        
        let mut surface = Surface::new(generated.as_mut_slice(), size.width, size.height);
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        surface.clear(0);
        func(&mut surface);
        print(&generated, &str_to_bitmap(expected), size);
        compare(&generated, &str_to_bitmap(expected))
    }
}

#[cfg(feature = "dom")]
pub struct DOMTest {}
#[cfg(feature = "dom")]
impl DOMTest {
    pub fn run<F>(size: Size, mut func:F, expected: &str) -> bool where F: FnMut(&mut DOM) {
        let mut generated = std::vec![0u8;(size.width as usize * size.height as usize + 7) / 8];        
        let mut surface = Surface::new(generated.as_mut_slice(), size.width, size.height);
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        let mut dom = DOM::new();
        func(&mut dom);
        surface.update(&mut dom);
        print(&generated, &str_to_bitmap(expected), size);
        compare(&generated, &str_to_bitmap(expected))
    }
}

fn print(generated_image: &[u8], expected_image: &[u8], size: Size) {
    let mut outputs = [std::vec![], std::vec![]];    

    let mut p = Writer::new();
    p    
    //.style(Style::ASCII1x1('#'))
    .style(Style::UnicodeBlock1x2)    
    .frame(Frame::UnicodeFrame)
    .byte_aligned(false)
    .ansi_position_restore(false);
    
    p.write(&mut outputs[0], &Bitmap::new(size.width.into(), size.height.into(), &generated_image));
    p.write(&mut outputs[1], &Bitmap::new(size.width.into(), size.height.into(), &expected_image));
    
    let mut lines = [str::from_utf8(&outputs[0]).unwrap().lines(), str::from_utf8(&outputs[1]).unwrap().lines()];
    
    std::println!("");
    //for _ in 0..size.height {
    for _ in 0..(2 + (size.height + 1) / 2) {        
        std::println!("{} {}", lines[0].next().unwrap_or(""), lines[1].next().unwrap_or(""));
    }
    let width = size.width as usize + 2;
    std::println!("{:^width$} {:^width$}", "generated", "expected");    
}

fn compare(generated_image: &[u8], expected_image: &[u8]) -> bool {
    return generated_image == expected_image;
}

pub fn str_to_bitmap(str: &str) -> std::vec::Vec::<u8> {
    let mut v = std::vec![];        
    for (i, c) in str.chars().into_iter().enumerate() {        
        if i % 8 == 0 { v.push(0); }
        if c != ' ' { v[i / 8] |= 1 << (7 - (i % 8)); }
    }
    return v;
}