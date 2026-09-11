use crate::{*, surface::Surface, media::{ Media, Identifier }, control::{ Control, IControl, IControlLoader }};

#[repr(u8)]
enum Ret {
    Ok = 0,
    IllegalCommand = 1
}

impl IControl for Surface {    
    fn process_line(&mut self, line: &str, _media: &Media, res: &mut dyn core::fmt::Write) -> (bool, core::fmt::Result) {
        let prefix = "@";
        if line.starts_with(prefix){
            let surface = self;                   
            let mut split = line[prefix.len()..].split(",");
            let cmd = split.next();
            let mut text = "";            
            let values = [0;8].map(|_| {
                let item = split.next();
                if item.is_some() { text = item.unwrap() };
                item.unwrap_or("0").parse().unwrap_or(0)
            });            
            let mut ret = Ret::Ok;
            match cmd.unwrap_or("").trim() {
                "c" => { surface.clear(values[0] as Color); }
                "p" => { surface.draw_pixel(Point{ x: values[0] as PosX, y: values[1] as PosY }, values[2] as Color); }
                "r" => { surface.draw_rect(Area { point: Point{ x: values[0] as PosX, y: values[1] as PosY }, size : Size { width: values[2], height: values[3] } }, values[4] as Color); }
                "f" => { surface.fill_rect(Area { point: Point{ x: values[0] as PosX, y: values[1] as PosY }, size : Size { width: values[2], height: values[3] } }, values[4] as Color); }
                "t" => { surface.draw_text(Identifier::Index(values[2] as u8), text, Point { x: values[0] as PosX, y: values[1] as PosY }); }
                "T" => { let size = surface.get_text_size(Identifier::Index(values[0] as u8), text); return (true, res.write_fmt(format_args!("{},{}", size.width, size.height))); }
                "i" => { surface.draw_image(Identifier::Both(values[2] as u8, text), Point { x: values[0] as PosX, y: values[1] as PosY }, None); }
                "b" => { surface.draw_border(values[4] as u8, Area { point: Point{ x: values[0] as PosX, y: values[1] as PosY }, size : Size { width: values[2], height: values[3] } } ); }
                "l" => { surface.draw_line(Point{ x: values[0] as PosX, y: values[1] as PosY }, Point{ x: values[2] as PosX, y: values[3] as PosY }, values[4] as u8); },                
                "C" => { surface.palette[0] = core::cmp::min(values[0], u8::MAX as u16) as u8; surface.palette[1] = core::cmp::min(values[1], u8::MAX as u16) as u8; }
                "init" => { surface.clear(0); surface.media.clear(); }                                
                #[cfg(feature = "inspect")]
                "inspect" => { return (true, surface.media.inspect(255, res)); }
                _ => { ret = Ret::IllegalCommand; }
            }
            Control::send_result(prefix, ret as u8, res)
        } else{
            (false, Ok(()))               
        }
    }

    fn get_loader(&mut self) -> Option<&mut dyn IControlLoader> { Some(self) }
}
