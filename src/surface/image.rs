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
                if image.mask.is_none(){                
                    self.render_stream(format, width, height, &image.data[IMAGE_HEADER_SIZE ..], position, drawing.inverted, &drawing.colors);
                }else{                                        
                    self.render_masked_stream(format, width, height, &image.data[IMAGE_HEADER_SIZE ..], &image.mask.unwrap(), position, drawing.inverted, &drawing.colors, drawing.use_mask);
                }
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

    pub fn render_stream(&self, _format: ImageFormat, width: ImageW, height: ImageH, data: &[u8], position: Point, inverted: bool, colors: &[ColorType]) {
        let mut p = position;
        let mut r_i = 0;    

        let mut reader: BinaryStream<'_> = BinaryStream::new(data);    
        for _p_i in 0 .. width as usize * height as usize {
            let pixel = reader.read_bit();
            if reader.ok == false {
                break;
            }
            
            if pixel == 1 {            
                let mut color = if (pixel as usize) < colors.len() { colors[pixel as usize] } else { Color::TRANSPARENT };
                if inverted == true {
                    color = Color::invert(color);
                }                              
                if colors[0] != color {                                                            
                    self.draw_pixel(p, color);
                }        
            }

            r_i += 1;
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
        }
    }

    pub fn render_masked_stream(&self, format: ImageFormat, width: ImageW, height: ImageH, image_data: &[u8], mask_data: &[u8], position: Point, inverted: bool, colors: &[ColorType], use_mask: bool) {
        let mut p = position;
        let mut r_i = 0;    
        
        let mut image_reader = Stream::new(&image_data[1..]);
        let mut image_reader_bit = BinaryStream::new(image_data);        
        let mut mask_reader = BinaryStream::new(&mask_data);        

        for _p_i in 0 .. width as usize * height as usize {            
            let masked = mask_reader.read_bit();
            if mask_reader.ok == false {
                break;
            }
            
            let mut color: ColorType;
            if format == ImageType::Indexed1 as ImageFormat {
                let pixel = image_reader_bit.read_bit();        
                if image_reader_bit.ok == false {
                    break;
                }
                color = if (pixel as usize) < colors.len() { colors[pixel as usize] } else { Color::TRANSPARENT };
            }else
            if format == ImageType::RGB888 as ImageFormat {
                let r = image_reader.read_u8();
                let g = image_reader.read_u8();
                let b = image_reader.read_u8();
                if image_reader.ok == false {
                    break;
                }
                color = Color::from_rgb(r, g, b);                
            }else{
                break;
            }                                 

            if !use_mask || masked == 1 {                
                if inverted == true {
                    color = Color::invert(color);
                }   
                self.draw_pixel(p, color);
            }
                                
            r_i += 1;
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
            
        }
    }

}