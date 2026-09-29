use crate::*;
use crate::surface::*;
use crate::media::*;
use crate::control::*;
#[cfg(feature = "vm")]
use virtmach::{self, VirtMach, interrupts::{ self, SoftInterrupt }};
#[cfg(feature = "vm")]
use crate::int_surface::IntSurface;

struct RuntimePrivate {    
}

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

    #[cfg(feature = "dom")]
    fn _intro_with_dom (surface: &mut Surface, dom: &mut dom::DOM, loop_cnt: usize) -> bool {
        match loop_cnt {                     
            0       => { dom.from(surface.media.get_page(SYS_LOGO).content); dom.node_ref_mut(2).set_height(0); }
            1 .. 64 => { dom.node_ref_mut(2).set_height(loop_cnt as SizeH); }            
            72      => { dom.from(surface.media.get_page(0).content); }
            _ => {  return false; }
        }    
        true
    }

    fn popup (surface: &mut Surface, message: &str) {                    
        let font = 0;        
        let text_size = surface.get_text_size(Identifier::Index(font), message);
        let size = Size { width: surface.width, height: text_size.height.saturating_add(8) };
        //let point = Point { x: (surface.width / 2).saturating_sub(size.width / 2) as PosX, y: (surface.height / 2).saturating_sub(size.height / 2) as PosY };
        let point = Point { x: (surface.width / 2).saturating_sub(size.width / 2) as PosX, y: (surface.height).saturating_sub(size.height) as PosY };
        surface.fill_rect(Area { point: Point { x: point.x, y: point.y }, size: Size { width: size.width, height: size.height } }, 0);
        surface.draw_rect(Area { point: Point { x: point.x + 1, y: point.y + 1 }, size: Size { width: size.width - 2, height: size.height - 2 } }, 1);        
        surface.draw_text(media::Identifier::Index(font), message, Point { x: 4, y: point.y + 4 });
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

pub struct DOM <'a>{
    loop_cnt: usize,
    pub status_message: Option<&'a str>,
    
    pub surface: Surface,
    #[cfg(feature = "dom")]
    pub dom: dom::DOM,    
    pub control: Option<Control>,

    #[cfg(feature = "vm")]
    pub vm: VirtMach<'a>,    
    #[cfg(feature = "vm")]
    vm_clip: Clip,
}

impl DOM<'_> {
    pub fn new(buf: & mut [u8], width: SizeW, height: SizeH) -> Self {
        Self {
            loop_cnt: 0,
            status_message: None,
            surface: Surface::new(buf, width, height),
            #[cfg(feature = "dom")]
            dom: dom::DOM::default(),
            control: Some(Control::default()),
            #[cfg(feature = "vm")]
            vm: VirtMach::new(),
            #[cfg(feature = "vm")]
            vm_clip: Clip { p0: Point { x: 0, y: 0 }, p1: Point { x: width as PosX, y: height as PosY } },
        }
    }    
}

impl Runtime for DOM<'_> {
    fn run(&mut self) -> bool {
        if self.loop_cnt == 0 {
            self.surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        }

        if !RuntimePrivate::intro(&mut self.surface, self.loop_cnt) {                 
         
            if self.control.is_some() && self.control.as_ref().unwrap().is_loader_busy() {
                let control = self.control.as_mut().unwrap();
                self.surface.draw_progress_screen(control.get_loader_progress(255).take().unwrap().try_into().unwrap(), control.get_loader_error() as u8);            
            }else{
                let clip = self.surface.clip;
                #[cfg(feature = "vm")]
                {                    
                    if self.vm.cycle_cnt == 0 {
                        self.vm_clip.reset(self.surface.width, self.surface.height);
                    }
                    self.surface.clip = self.vm_clip;

                    let int0: &mut dyn SoftInterrupt = &mut interrupts::proc::Interrupt {};
                    let int1: &mut dyn SoftInterrupt = &mut interrupts::math::Interrupt {};
                    let int2: &mut dyn SoftInterrupt = &mut interrupts::string::Interrupt {};
                    let int3: &mut dyn SoftInterrupt = &mut interrupts::random::Interrupt {};        
                    let int4: &mut dyn SoftInterrupt = &mut IntSurface { surface: &mut self.surface };
                    let mut interrupts = [int0, int1, int2, int3, int4];                    
                                        
                    self.vm.run(1024, &mut interrupts);
                    self.vm_clip = self.surface.clip;
                    self.surface.clip.reset(self.surface.width, self.surface.height);
                }
                #[cfg(feature = "dom")]
                {
                    #[cfg(feature = "vm")]
                    match self.vm.state {
                        virtmach::Runtime::Ini | virtmach::Runtime::Stp => {
                            self.surface.clear(0);
                            self.surface.update(&mut self.dom); 
                        }
                        _ => {}
                    }
                    #[cfg(not(feature = "vm"))]
                    {
                        self.surface.clear(0);
                        self.surface.update(&mut self.dom); 
                    }
                }
                self.surface.clip = clip;
                
                // let x = (self.surface.width as f32 / 2.0) + (((self.loop_cnt as f32) / 10.0).cos() * self.surface.width as f32 / 2.5);
                // let y = (self.surface.height as f32 / 2.0) + (((self.loop_cnt as f32) / 10.0).sin() * self.surface.height as f32 / 2.5);
                let x = 16.0 + (libm::cosf((self.loop_cnt as f32) / 10.0) * 16.0);
                let y = 0.0 + (libm::sinf((self.loop_cnt as f32) / 17.0) * 8.0);
                RuntimePrivate::cursor(&mut self.surface, Point { x: x as PosX, y: y as PosY });
            }

            RuntimePrivate::demo_popup(&mut self.surface, self.loop_cnt);
            
            if self.status_message.is_some() {
                RuntimePrivate::popup(&mut self.surface, self.status_message.unwrap());            
            }
        }

        self.loop_cnt = self.loop_cnt.checked_add(1).unwrap_or(self.surface.height as usize + RuntimePrivate::INTRO_WAIT);       

        true
    }

    fn run_cnt(&self) -> usize {
        return self.loop_cnt;
    }

    fn process_command(&mut self, input: &[u8], mut res: &mut dyn core::fmt::Write, external_listener: &mut dyn IControl) -> (usize, core::fmt::Result) {
        if self.control.is_some() {            
            self.control.as_mut().unwrap().process(input, &mut self.surface, &mut [
                Some(external_listener),
                #[cfg(not(feature = "dom"))]
                None,
                #[cfg(feature = "dom")]
                Some(&mut self.dom),
                #[cfg(not(feature = "vm"))]
                None,
                 #[cfg(feature = "vm")]
                Some(&mut self.vm),
                None
            ], &mut res)           
        }else{
            log::error!("no listener available");
            (input.len(), Ok(()))
        }        
    }
}