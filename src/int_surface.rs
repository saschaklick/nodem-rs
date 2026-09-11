use virtmach::{ *, interrupts::{ SoftInterrupt } };
    #[cfg(feature = "compile")]
use virtmach::{ interrupts::{ surface, SoftInterruptFunction } };
use crate::surface::Surface;
use crate::{ Area, Size, Point, PosX, PosY, SizeW, SizeH, Color, BorderIdx, media::Identifier };

pub struct IntSurface <'a> {    
    pub surface: &'a mut Surface
}

impl SoftInterrupt for IntSurface <'_> {
    fn name(&self) -> &str {
        return "surface";
    }
    
    #[cfg(feature = "compile")]
    fn functions(&self) -> &'static [SoftInterruptFunction<'static>] where Self:Sized {
        return &surface::FUNCTIONS;
    }
    
    fn call(&mut self, vm: &mut VirtMach) {                
        let op = vm.stack_pop();        
        
        match op {
            0 => {
                let color = vm.stack_pop() as u8;                
                self.surface.clear(color);
            }     
            1 | 6 => {
                let point = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                let value = vm.stack_pop();                
                match op {
                    1 => { self.surface.draw_pixel(point, value as Color); }
                    _ => { self.surface.draw_image(Identifier::Index(value as u8), point, None); }
                }
            }    
            2 | 3 | 5 => {
                let area = Area { point: Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY }, size: Size { width: vm.stack_pop() as SizeW, height: vm.stack_pop() as SizeH } };                
                let value = vm.stack_pop();                
                match op {
                    2 => self.surface.draw_rect(area, value as Color),
                    3 => self.surface.fill_rect(area, value as Color),                    
                    _ => self.surface.draw_border(value as BorderIdx, area)
                }
            }
            4 | 18 => {
                let p_0 = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                let p_1 = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                match op {
                    4 => {
                        let color = vm.stack_pop() as u8;                
                        self.surface.draw_line(p_0, p_1, color);
                    },
                    _ => self.surface.set_clip(p_0, p_1)
                    
                }
                
            }    
            16 => {
                vm.stack_push(self.surface.width as VMAtom);
                vm.stack_push(self.surface.height as VMAtom);
            } 
            17 => {
                // vm.stack_pop();
                // vm.stack_push(0);
                // vm.stack_push(0);
            }                       
            19 => {
                self.surface.clip.p0.x = vm.stack_pop() as PosX;
                self.surface.clip.p0.y = vm.stack_pop() as PosY;
                self.surface.clip.p1.x = vm.stack_pop() as PosX;
                self.surface.clip.p0.y = vm.stack_pop() as PosY;
            }                               
            _ => { vm.error = RuntimeError::UnimplementedInterruptFunc; }
        }        
    }

}
