use nodem_rs::*;
use bitmap_writer::{Style, Frame, Bitmap, Writer};
use std::io::BufWriter;
use simple_logger::SimpleLogger;

pub struct Example {
    #[allow(dead_code)]
    pub dom: nodem_rs::dom::DOM
}
impl Example {
    #[allow(dead_code)]
    pub fn new() -> Self {
        return Self {
            dom: Default::default()
        }
    }
    
    #[allow(dead_code)]
    pub async fn main() {
        SimpleLogger::new().init().unwrap();

        Terminal::new(Example::new()).run();
    }
}

pub struct Terminal {
    example: Example
}
impl Terminal {
    pub fn new(example: Example) -> Self {
        let mut app = Self {
            example: example
        }; 
        app.run();
        return app;
    }

    pub fn run(&mut self) {
        let mut pixels: [u8; SCREEN_BUFFER_SIZE] = [0b00000000; SCREEN_BUFFER_SIZE];
        
        let mut surface = surface::Surface::new(&mut pixels, SCREEN_WIDTH, SCREEN_HEIGHT);

        self.example.init(&mut surface);        
        
        let mut loop_cnt: u32 = 0;
        loop {
            if loop_cnt != 0 { break; }
        
            let cursor = Point { x: 35 + (((loop_cnt as f32) / 10.0).cos() * 40.0) as PosX, y: 3 + (((loop_cnt as f32) / 17.0).cos() * 5.0) as PosY };

            self.example.repeat(&mut surface, loop_cnt, cursor, 0);
            
            let image = Bitmap::new(surface.width.into(), surface.height.into(), &pixels);

            let mut buf = BufWriter::new(Vec::new());           
            
            let mut p = Writer::new();
            p.style(Style::UnicodeBlock1x2)
            .frame(Frame::UnicodeDoubleUFrame)
            .ansi_position_restore(true)
            .write(&mut buf, &image);

            print!("{}", String::from_utf8(buf.into_inner().unwrap()).unwrap());

            let wait: u64 = unsafe { core::arch::x86_64::_rdtsc() } + 100000000;
            loop {
                if wait < unsafe { core::arch::x86_64::_rdtsc() } { break; }
            }
                        
            loop_cnt += 1;            
        }
    }
}
