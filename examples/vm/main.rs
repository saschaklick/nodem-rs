use bytes::BytesMut;
use cfg_block::cfg_block;
use nodem_rs::int_surface::{ IntSurface };
use nodem_rs::media::{ font::Alignment, Identifier };
use virtmach::{self, VirtMach, Program, interrupts::{ self, SoftInterrupt }};

cfg_block! {
    if #[cfg(feature = "ui")] {
        #[path = "../../impl/sdl2.rs"]
        mod sdl2;
        use sdl2::Example;
    } else {
        #[path = "../../impl/term.rs"]        
        mod term;
        use term::Example;
    }
}

use nodem_rs::*;
use nodem_rs::surface::*;
    
pub static PKG_SYS: &'static [u8] = include_bytes!("../../../nodem-pkg/pkg/sys.pkg");

static PROGRAM: &str = "     
    ; Count up register 0 and memory address 0
    start:
        reg r0
        add #1
        sto #0
        hlt
        jmp start
";


static EMPTY: [u8;0] = [];
static mut VM: Option<VirtMach> = None;
static mut PAUSED: bool = true;
static mut RUN_TO_HLT: bool = false;
static mut VM_MONITOR: bool = false;
static mut SHOW_CONTROLS: bool = true;

impl Example {
    pub fn init(self: &mut Self, surface: &mut Surface) {
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        
        let mut vm = VirtMach::new();          
                                
        let res = VirtMach::compile("PROGRAM", PROGRAM, vec![
            (interrupts::proc::NAME, interrupts::proc::FUNCTIONS.as_slice()),
            (interrupts::math::NAME, interrupts::math::FUNCTIONS.as_slice()),
            (interrupts::random::NAME, interrupts::random::FUNCTIONS.as_slice()),
            (interrupts::surface::NAME, interrupts::surface::FUNCTIONS.as_slice())
        ]);

        match res {
            Ok(res) => {                    
                let mut buf = BytesMut::new();
                let mut pos = 0usize;
                loop {
                    let addr = pos;
                    pos = VirtMach::decompile(&res.0, pos, &mut buf);
                    log::info!("| {:04x} | {}", addr, str::from_utf8(&buf).unwrap());
                    buf.clear();

                    if pos >= res.0.data.len() { break }
                }
                
                vm.load_program(res.0);                                    
            }
            _ => {                             
                log::error!("compile error: {:?}", res.err().unwrap());
                vm.load_program(Program::ERROR);
            }
        }

        vm.load_program(surface.media.get_program(3));                    

        unsafe { VM = Some(vm); }        
    }

    pub fn repeat(self: &mut Self, surface: &mut Surface, _repeat_cnt: u32, cursor: Point, action: u8) {        
        let mut vm = unsafe { VM.take().unwrap() };                

        fn print_report(vm: &VirtMach) {
            let mut buf = BytesMut::with_capacity(1024);
            vm.write_dashboard(&mut buf, 0b111, 5);
            print!("\x1b[H\x1b[J");          
            println!("{}", str::from_utf8(&buf).unwrap());
        }

        let int0: &mut dyn SoftInterrupt = &mut interrupts::proc::Interrupt {};
        let int1: &mut dyn SoftInterrupt = &mut interrupts::math::Interrupt {};
        let int2: &mut dyn SoftInterrupt = &mut interrupts::random::Interrupt {};
        let int3: &mut dyn SoftInterrupt = &mut IntSurface { surface: surface };
        let mut interrupts = [int0, int1, int2, int3];

        if !unsafe { PAUSED } || unsafe { RUN_TO_HLT } {
            vm.run(1024, &mut interrupts);                    
            //vm.log();        

            if (unsafe { RUN_TO_HLT } == true) && !vm.running() {                
                unsafe{ PAUSED = true; RUN_TO_HLT = false; };
            }
            
            print_report(&vm);
        }        

        let mut run_to_halt = false;
        match action {
            1 => {
                if unsafe { SHOW_CONTROLS }{
                    if cursor.x >= 72 && cursor.x < 98  && cursor.y >= 43 && cursor.y < 53 { unsafe { PAUSED = !PAUSED; } }
                    if cursor.x >= 72 && cursor.x < 98  && cursor.y >= 53 && cursor.y < 63 { unsafe { RUN_TO_HLT = true }; }
                    if cursor.x >= 98 && cursor.x < 127 && cursor.y >= 43 && cursor.y < 53 { if unsafe { !PAUSED } { unsafe { PAUSED = true; } } else { vm.run(1, &mut interrupts); print_report(&vm); } }
                    if cursor.x >= 98 && cursor.x < 127 && cursor.y >= 53 && cursor.y < 58 { unsafe { VM_MONITOR = !VM_MONITOR; } }
                    if cursor.x >= 98 && cursor.x < 127 && cursor.y >= 58 && cursor.y < 63 { vm.reset(); }
                    if cursor.x >= 123 && cursor.x < 127 && cursor.y >= 38 && cursor.y < 43 { unsafe { SHOW_CONTROLS = !SHOW_CONTROLS; } }
                }else{
                    unsafe { SHOW_CONTROLS = !SHOW_CONTROLS; }
                }
            }
            _ => {}
        }
        
        let mut buf = String::new();
        vm.write_dashboard(&mut buf, 0b11,  5);
                
        let controls = format!(concat!(
            "@CONTROLS@@@@x\n",
            "[{0:}][STEP ]\n",
            "[{0:}][STEP ]\n",            
            "[HALT ][DEBUG]\n",
            "[HALT ][RESET]"), if unsafe { PAUSED } { "RUN  " } else { "PAUSE" });

        surface.media.typesetting.mono_spaced = Alignment::Center;                
        if unsafe { VM_MONITOR } {
            surface.clear(0);    
            surface.draw_text(Identifier::Index(0), &buf.to_ascii_uppercase(), Point { x: 0, y: 0 });
        }
        if unsafe { SHOW_CONTROLS } {
            surface.fill_rect(Area { point: Point { x: 72 - 1, y: 39 - 1}, size: Size { width: 128 - 72 + 1, height: 64 - 39 + 1 } }, 0);
            surface.draw_text(Identifier::Index(0), &controls.trim(), Point { x: 72, y: 39 });        
        }

        // surface.clear(0);    
        // surface.fullscreen(&mut self.dom);
        // surface.update(&self.dom);
                
        surface.draw_image(Identifier::Index(252), cursor, None);              

        unsafe { VM = Some(vm); }
    }
}

#[tokio::main]
async fn main(){    
    Example::main().await;
}