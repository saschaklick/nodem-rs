use crate::*;
use crate::surface::*;
use crate::media::*;
use crate::control::*;
#[cfg(feature = "vm")]
use virtmach::VirtMach;

struct RuntimePrivate {}

impl RuntimePrivate {
    const INTRO_WAIT: usize = 16;

    fn intro (surface: &mut Surface, loop_cnt: usize) -> bool {                
        if loop_cnt < surface.height as usize + RuntimePrivate::INTRO_WAIT {
            if loop_cnt == 0 || (loop_cnt > RuntimePrivate::INTRO_WAIT && loop_cnt < surface.height as usize + RuntimePrivate::INTRO_WAIT) {
                let size = surface.get_image_size(Identifier::Index(SYS_LOGO));
                let y = if loop_cnt == 0 { 0 } else { (loop_cnt - RuntimePrivate::INTRO_WAIT) as PosY };
                surface.clear(0);
                surface.draw_image(Identifier::Index(SYS_LOGO), Point { x: (surface.width as PosX - size.width as PosX) / 2, y: (surface.height as PosY - size.height as PosY) / 2 + y }, None);     
            }        
            true
        }else{
            false
        }
    }

    fn _intro_with_dom (surface: &mut Surface, dom: &mut dom::DOM, loop_cnt: usize) -> bool {
        match loop_cnt {                     
            0       => { dom.from(surface.media.get_page(SYS_LOGO).content); dom.node_ref_mut(2).set_height(0); }
            1 .. 64 => { dom.node_ref_mut(2).set_height(loop_cnt as SizeH); }            
            72      => { dom.from(surface.media.get_page(0).content); }
            _ => {  return false; }
        }    
        true
    }

    fn demo_popup (surface: &mut Surface, loop_cnt: usize) {            
        if (loop_cnt % 400) > 350 {
            let text = "DEMO";
            let size = surface.get_text_size(Identifier::Index(0), text);
            surface.fill_rect(Area { point: Point { x: 0, y: 0 }, size: Size { width: size.width + 1, height: size.height + 1 } }, 0);        
            surface.draw_text(Identifier::Index(0), text, Point { x: 0, y: 0 });
        }
    }

    fn cursor (surface: &mut Surface, point: Point) {
        surface.draw_image(Identifier::Index(SYS_POINTER), point, None);
    }        
}

pub static PKG_SYS: &'static [u8] = include_bytes!("sys.pkg");

pub trait Runtime {
    fn run(&mut self) -> bool;

    fn run_cnt(&self) -> usize;

    fn process_command(&mut self, input: &[u8], res: &mut dyn core::fmt::Write, external_listener: &mut dyn IControl) -> (usize, core::fmt::Result);
}

pub struct DOM {
    loop_cnt: usize,    
    
    pub surface: Surface,
    pub dom: dom::DOM,    
    pub control: Option<Control>
}
impl DOM {
    pub fn new(buf: & mut [u8], width: SizeW, height: SizeH) -> Self {
        Self {
            loop_cnt: 0,
            surface: Surface::new(buf, width, height),
            dom: dom::DOM::default(),
            control: Some(Control::default())
        }
    }    
}

impl Runtime for DOM {
    fn run(&mut self) -> bool {
        if self.loop_cnt == 0 {
            self.surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        }

        if !RuntimePrivate::intro(&mut self.surface, self.loop_cnt) {                 
         
            if self.control.is_some() && self.control.as_ref().unwrap().is_loader_busy() {
                let control = self.control.as_mut().unwrap();
                self.surface.draw_progress_screen(control.get_loader_progress(255).take().unwrap().try_into().unwrap(), control.get_loader_error());            
            }else{
                self.surface.clear(0);
                self.surface.update(&mut self.dom); 
                // let x = (self.surface.width as f32 / 2.0) + (((self.loop_cnt as f32) / 10.0).cos() * self.surface.width as f32 / 2.5);
                // let y = (self.surface.height as f32 / 2.0) + (((self.loop_cnt as f32) / 10.0).sin() * self.surface.height as f32 / 2.5);
                let x = 16.0 + (((self.loop_cnt as f32) / 10.0).cos() * 16.0);
                let y = 0.0 + (((self.loop_cnt as f32) / 17.0).sin() * 8.0);
                RuntimePrivate::cursor(&mut self.surface, Point { x: x as PosX, y: y as PosY });
            }

            RuntimePrivate::demo_popup(&mut self.surface, self.loop_cnt);
        }

        self.loop_cnt = self.loop_cnt.checked_add(1).unwrap_or(self.surface.height as usize + RuntimePrivate::INTRO_WAIT);       

        true
    }

    fn run_cnt(&self) -> usize {
        return self.loop_cnt;
    }

    fn process_command(&mut self, input: &[u8], mut res: &mut dyn core::fmt::Write, external_listener: &mut dyn IControl) -> (usize, core::fmt::Result) {
        if self.control.is_some() {
            let ret = self.control.as_mut().unwrap().process(input, &mut self.surface, &mut [Some(&mut self.dom), Some(external_listener), None, None], &mut res);
            ret
        }else{
            log::error!("no control available");
            (input.len(), Ok(()))
        }        
    }
}