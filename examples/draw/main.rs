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

use nostd_structs::algos::rand::lcg::LcgRng;
use std::time::{SystemTime};

use nodem_rs::*;
use nodem_rs::media::Identifier;
use nodem_rs::surface::*;

pub static PKG_SYS: &'static [u8] = include_bytes!("../../pkg/sys.pkg");

static mut POINT: Point = Point { x: 0, y: 0 };
static mut FPS_CNT: u32 = 0;
static mut FPS: u32 = 0;
static mut HEARTBEAT: SystemTime = SystemTime::UNIX_EPOCH;

impl Example {
    pub fn init(self: &mut Self, surface: &mut Surface) {
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        unsafe { HEARTBEAT = SystemTime::now(); }
    }

    pub fn repeat(self: &mut Self, surface: &mut Surface, _repeat_cnt: u32, cursor: Point, _action: u8) {
        let mut rng = LcgRng::new(0);
        
        let p_last = unsafe { POINT };
        let p_next = Point { x: (rng.next() as f32 / u64::MAX as f32 * (surface.width as f32 - 1.0)) as PosX, y: (rng.next() as f32 / u64::MAX as f32 * (surface.height as f32 - 1.0)) as PosY };
        surface.draw_line(p_last, p_next, 1 as Color);        
        surface.draw_line(p_last + Point { x: 1, y: 1 }, p_next + Point { x : 1, y: 1 }, 0 as Color);
        unsafe { POINT = p_next; }

        surface.draw_line(Point { x: (surface.width / 2) as PosX, y: (surface.height / 2) as PosY }, cursor, 1);

        unsafe {
            FPS_CNT += 1;
            let _ = HEARTBEAT.elapsed().inspect(|duration| {
                if duration.as_secs() >= 1 {
                    FPS = FPS_CNT;
                    FPS_CNT = 0;
                    HEARTBEAT = SystemTime::now();
                }
            });
        }
        

        let fps_text = format!("{:3}", unsafe { FPS });
        let fps_size = surface.get_text_size(Identifier::Index(0), fps_text.as_str()) + Size { width: 5, height: 5 };
        let fps_area = Area { point: Point { x: 0, y: 0 }, size: fps_size };
        surface.fill_rect(fps_area, 0);                
        surface.draw_rect(Area { point: fps_area.point, size : fps_area.size - Size { width: 1, height: 1 } }, 1);                
        surface.draw_text(Identifier::Index(0), &fps_text, Point { x: 2, y: 2 });

    }
}

#[tokio::main]
async fn main(){    
    Example::main().await;
}