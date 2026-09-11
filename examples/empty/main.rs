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
use nodem_rs::surface::*;

pub static PKG_SYS: &'static [u8] = include_bytes!("../../pkg/sys.pkg");

impl Example {
    pub fn init(self: &mut Self, surface: &mut Surface) {
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
    }

    pub fn repeat(self: &mut Self, _surface: &mut Surface, _repeat_cnt: u32, _cursor: Point, _action: u8) {

    }
}

#[tokio::main]
async fn main(){    
    Example::main().await;
}