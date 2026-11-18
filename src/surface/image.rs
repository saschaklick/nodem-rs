use crate::surface::*;
use crate::stream::*;
use crate::media::image::*;

impl Surface {
    pub fn draw_image(&self, ident: Identifier, position: Point, drawing: Option<Settings>) -> Size {
        let image = self.media.get_image(ident);    
        let mut reader = Stream::new(image.data);
        let drawing = drawing.unwrap_or(self.media.drawing);        

        if reader.available() >= 3 {
            let format = reader.read_u8();
            let width = reader.read_u8();
            let height = reader.read_u8();

            if format == ImageType::Dummy as u8 {
                self.draw_rect(Area { point: position, size: Size { width: width as SizeW, height: height as SizeH } }, 1);
            }else{                
                self.render_stream(width, height, &image.data[IMAGE_HEADER_SIZE ..], if drawing.use_mask && image.mask.is_some() { Some(&image.mask.unwrap()) } else { None }, position, drawing.inverted, &drawing.colors);
            }
            return Size { width: width as SizeW, height: height as SizeH }
        }else{
            return Size::default();
        }
    }

    pub fn get_image_size(&self, ident: Identifier) -> Size {
        let image = self.media.get_image(ident);
        let mut reader = Stream::new(image.data);

        if reader.available() >= 3 {
            let _format = reader.read_u8();
            let width = reader.read_u8();
            let height = reader.read_u8();

            return Size { width: width as SizeW, height: height as SizeH }
        }else{
            return Size::default();
        }
    }

    pub fn render_stream(&self, width: ImageW, height: ImageH, image_data: &[u8], mask_data: Option<&[u8]>, position: Point, inverted: bool, colors: &[ColorType]) {
        let mut p = position;
        let mut r_i = 0;            
        
        let mut pixel_reader = PixelStream::new(image_data, &colors);        
        let mut mask_reader = BitStream::new(&mask_data.unwrap_or(&[0u8;0]));        

        let mut masked = 1;
        for _p_i in 0 .. width as usize * height as usize {            
            if mask_data.is_some() {
                masked = mask_reader.read_bit();
                if mask_reader.ok == false {
                   break;
                }
            }
            
            let mut color = pixel_reader.next();        
            if pixel_reader.ok == false {
                break;
            }
            
            if masked == 1 && color != Color::TRANSPARENT {
                if inverted == true {
                    color = Color::invert(color);
                }   
                self.draw_pixel(p, color);
            }
                                
            r_i += 1;
            if true {                
                if r_i == width {
                    r_i = 0;
                    p.x = position.x;
                    if p.y >= self.height as PosY {
                        break;
                    }
                    p.y += 1;
                }else{
                    p.x += 1;
                }
            } else {
                if r_i == width {
                    r_i = 0;
                    p.y = position.y;
                    if p.x >= self.width as PosX {
                        break;
                    }
                    p.x += 1;
                }else{
                    p.y += 1;
                }
            }
        }
    }

}