use nodem_rs::*;
use nodem_rs::control::*;
use std::ptr::addr_of_mut;
use bytes::BytesMut;

use simple_logger::SimpleLogger;
use sdl2;
use sdl2::pixels::{Color, Palette};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Duration;
use tokio::{ net::TcpListener, io::AsyncWriteExt };

static SCALE: u32          = 5;
static WINDOW_PADDING: u32 = 64;

pub struct Example {    
    #[allow(dead_code)]
    #[cfg(feature = "dom")]
    pub dom: nodem_rs::dom::DOM
}
impl Example {
    pub fn new() -> Self {
        return Self {
            #[cfg(feature = "dom")]
            dom: Default::default()
        }
    }
    
    #[allow(dead_code)]
    pub async fn main() {
        SimpleLogger::new().init().unwrap();        

        SDL2Window::new(Example::new()).await;
    }
}

pub struct SDL2Window {
    example: Example
}

impl SDL2Window {
    pub async fn new(example: Example) -> Self {
        let mut app = Self {
            example: example
        }; 
        app.run().await;
        return app;
    }

    pub async fn run(self: &mut SDL2Window) {
        let control_port = 1234;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", control_port)).await.unwrap();        
        log::info!("listening on tcp port {}", control_port);

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let window = video_subsystem.window("nodem: sdl2-demo", SCREEN_WIDTH as u32 * SCALE + WINDOW_PADDING, SCREEN_HEIGHT as u32 * SCALE + WINDOW_PADDING)
            .position_centered()
            .build()
            .unwrap();   
            
        let mut pixels: [u8; SCREEN_BUFFER_SIZE] = [0b00000000; SCREEN_BUFFER_SIZE];
        
        let mut nodem_surface = surface::Surface::new(addr_of_mut!(pixels), SCREEN_WIDTH, SCREEN_HEIGHT);        

        let colors = [Color::RGB(32, 32, 32), Color::RGB(196, 196, 196)];
        let palette: Palette = Palette::with_colors(&colors).unwrap();

        self.example.init(&mut nodem_surface);

        let mut mouse_x: i32 = 0;
        let mut mouse_y: i32 = 0;
        let mut mouse_s: u8  = 0;
        let mut mouse_p: u8  = 0;

        let mut loop_cnt: u32 = 0;
        let mut s: Option<tokio::net::TcpStream> = None;

        let mut control = Control::default();        

        let mut buf = [0 as u8; 256];        

        'running: loop {
            let waker = std::task::Waker::noop();            
            
            let poll = listener.poll_accept(&mut std::task::Context::from_waker(&waker));
            if poll.is_ready(){
                let _ = poll.map_ok(|(stream, _addr)| {                                                            
                    s = Some(stream);            
                });
            }

            if s.is_some() {
                let mut stream = s.take().unwrap();                                
                                
                match stream.try_read(&mut buf) {                    
                    Ok(read_cnt) => {                                    
                        if read_cnt > 0 {                                                                
                            let mut res = BytesMut::with_capacity(1014);
                            #[cfg(feature = "dom")]
                            let _ = control.process(&buf[..read_cnt], &mut nodem_surface, &mut [Some(&mut self.example.dom), None, None, None], &mut res);                                                                                                                                            
                            #[cfg(not(feature = "dom"))]
                            let _ = control.process(&buf[..read_cnt], &mut nodem_surface, &mut [None, None, None, None], &mut res);                                                                                                                                            
                            if !res.is_empty() {                                                                
                                let _ = stream.write(&res).await;                                                                                              
                            }
                        }
                    }                    
                    Err(_err) => {   
                        //log::error!("failed to read stream: {}", err);
                    }
                }
                s = Some(stream);
            }            
            
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::MouseMotion { timestamp:_, window_id:_, which:_, mousestate:_, x, y, xrel:_, yrel:_} => {
                        mouse_x = x - WINDOW_PADDING as i32;
                        mouse_y = y - WINDOW_PADDING as i32;                        
                    }
                    Event::MouseButtonDown { timestamp:_, window_id:_, which:_, clicks:_, x, y, mouse_btn } => {
                        mouse_x = x - WINDOW_PADDING as i32;
                        mouse_y = y - WINDOW_PADDING as i32;
                        mouse_s = mouse_s | (mouse_btn as u8);                        
                    }
                    Event::MouseButtonUp { timestamp:_, window_id:_, which:_, clicks:_, x:_, y:_, mouse_btn } => {
                        mouse_s = mouse_s & !(mouse_btn as u8);
                    }
                    _ => {}
                }
            }            
            let cursor = Point { x: (mouse_x as i32 / SCALE as i32) as PosX, y: (mouse_y as i32 / SCALE as i32) as PosY };
            
            if control.is_loader_busy() {
                nodem_surface.draw_progress_screen(control.get_loader_progress(255).take().unwrap().try_into().unwrap(), control.get_loader_error() as u8);                
            }else{
                let action =
                    if mouse_s & 0b1 == 1 && mouse_p & 0b1 == 0 { 1 } else
                    if mouse_s & 0b1 == 0 && mouse_p & 0b1 == 1 { 2 } else { 0 };
                self.example.repeat(&mut nodem_surface, loop_cnt, cursor, action);                
            }
            mouse_p = mouse_s;
                
            let mut window_surface = window.surface(&event_pump).unwrap();      

            let mut surface: Surface = Surface::from_data( &mut pixels, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, SCREEN_WIDTH as u32 / 8, PixelFormatEnum::Index1MSB).unwrap();
            surface.set_palette(&palette).ok();  

            let r1 = Rect::new(0,0, surface.width(), surface.height() );
            let r2 = Rect::new(
                ((window_surface.width() - (surface.width() * SCALE)) / 2).try_into().unwrap(),
                ((window_surface.height() - (surface.height() * SCALE)) / 2).try_into().unwrap(),
                surface.width() * SCALE,
                surface.height() * SCALE
            );        
            surface.convert_format(window_surface.pixel_format().into()).unwrap().blit_scaled(r1, &mut window_surface, r2).ok();
            
            window_surface.update_window().ok();

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

            loop_cnt += 1;
        }
    }
}