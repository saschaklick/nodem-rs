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
            4 => {
                let p_0 = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                let p_1 = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                let color = vm.stack_pop() as u8;                
                self.surface.draw_line(p_0, p_1, color);
            }    
            10 => {
                let p = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };                
                let font_idx = vm.stack_pop();
                let db_idx = vm.stack_pop();
                self.surface.draw_text(Identifier::Index(font_idx as u8), vm.get_str(db_idx as u8), p);
            }
            15 => {
                let font_idx = vm.stack_pop();
                let db_idx = vm.stack_pop();
                let size = self.surface.get_text_size(Identifier::Index(font_idx as u8), vm.get_str(db_idx as u8));
                vm.stack_push(size.width as VMAtom);
                vm.stack_push(size.height as VMAtom);
            }
            16 => {
                vm.stack_push(self.surface.width as VMAtom);
                vm.stack_push(self.surface.height as VMAtom);
            } 
            17 => {
                let image_idx = vm.stack_pop();
                let size = self.surface.get_image_size(Identifier::Index(image_idx as u8));
                vm.stack_push(size.width as VMAtom);
                vm.stack_push(size.height as VMAtom);
            }
            // The VM side speaks (x,y,w,h), `Surface::clip` is two corners.
            18 => {
                let clip = self.surface.clip;
                vm.stack_push(clip.p0.x as VMAtom);
                vm.stack_push(clip.p0.y as VMAtom);
                vm.stack_push(clip.p1.x.saturating_sub(clip.p0.x) as VMAtom);
                vm.stack_push(clip.p1.y.saturating_sub(clip.p0.y) as VMAtom);
            }
            19 => {
                let p_0 = Point { x: vm.stack_pop() as PosX, y: vm.stack_pop() as PosY };
                let size = Size { width: vm.stack_pop() as SizeW, height: vm.stack_pop() as SizeH };
                let p_1 = Point { x: p_0.x.saturating_add_unsigned(size.width), y: p_0.y.saturating_add_unsigned(size.height) };
                self.surface.set_clip(p_0, p_1);
            }                               
            _ => { vm.error = RuntimeError::UnimplementedInterruptFunc; }
        }        
    }

}
