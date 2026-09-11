#[cfg(feature = "ui")]
#[path = "../../impl/sdl2.rs"]
#[cfg(feature = "ui")]
mod sdl2;
#[cfg(feature = "ui")]
use sdl2::Example;
#[cfg(not(feature = "ui"))]
#[path = "../../impl/term.rs"]        
#[cfg(not(feature = "ui"))]
mod term;
#[cfg(not(feature = "ui"))]
use term::Example;

use nodem_rs::*;
use nodem_rs::media::Identifier;
use nodem_rs::surface::*;
    
pub static PKG_SYS: &'static [u8] = include_bytes!("../../pkg/sys.pkg");
pub static PKG_INT: &'static [u8] = include_bytes!("../../pkg/int.pkg");

impl Example {
    pub fn init(self: &mut Self, surface: &mut Surface) {
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        surface.media.load_pkg(PKG_INT.as_ptr(), PKG_INT.len(), 1);                
    }

    pub fn repeat(self: &mut Self, surface: &mut Surface, repeat_cnt: u32, cursor: Point, _action: u8) {        
        match repeat_cnt {                        
            0  => { self.dom.from(surface.media.get_page(253).content); }
            0 .. 64 => { self.dom.node_ref_mut(2).set_height(repeat_cnt as SizeH); }
            72 => { self.dom.from(surface.media.get_page(1).content); }
            _ => {}
        }        

        surface.clear(0);
        surface.fullscreen(&mut self.dom);
        surface.update(&mut self.dom);
                
        surface.draw_image(Identifier::Index(252), cursor, None);              
    }
}

#[tokio::main]
async fn main(){    
    Example::main().await;
}