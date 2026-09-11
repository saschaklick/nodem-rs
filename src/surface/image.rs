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

            if format == ImageType::Dummy as ImageFormat {
                self.draw_rect(Area { point: position, size: Size { width: width as SizeW, height: height as SizeH } }, 1);
            }else{
                if image.mask.is_none(){                
                    self.render_stream(format, width, height, &image.data[IMAGE_HEADER_SIZE ..], position, drawing.inverted, drawing.transparent_color);
                }else{                                        
                    self.render_masked_stream(format, width, height, &image.data[IMAGE_HEADER_SIZE ..], &image.mask.unwrap(), position, drawing.inverted, drawing.use_mask);
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

    pub fn render_stream(&self, _format: ImageFormat, width: ImageW, height: ImageH, data: &[u8], position: Point, inverted: bool, transparent_color: Color) {
        let mut p = position;
        let mut r_i = 0;    

        let mut reader = BinaryStream::new(data);
            
        for _p_i in 0 .. width as usize * height as usize {
            let pixel = reader.read_bit();
            if reader.ok == false {
                break;
            }
            
            if transparent_color == 255 || transparent_color != pixel {                    
                let mut color_no = self.get_color(pixel as usize);
                if inverted == true {
                    color_no = if color_no == 0 { 1 } else { 0 };
                }
                let color = self.palette[color_no as usize];
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

    pub fn render_masked_stream(&self, _format: ImageFormat, width: ImageW, height: ImageH, image_data: &[u8], mask_data: &[u8], position: Point, inverted: bool, use_mask: bool) {
        let mut p = position;
        let mut r_i = 0;    
        
        let mut image_reader = BinaryStream::new(image_data);
        let mut mask_reader = BinaryStream::new(&mask_data);        
            
        for _p_i in 0 .. width as usize * height as usize {
            let pixel = image_reader.read_bit();        
            let masked = mask_reader.read_bit();
            if image_reader.ok == false || mask_reader.ok == false {
                break;
            }
                    
            let mut color_no = self.get_color(pixel as usize);
            if inverted == true {
                color_no = if color_no == 0 { 1 } else { 0 };
            }
            let color = self.palette[color_no as usize];

            if use_mask == true {
                if masked == 1 {                
                    self.draw_pixel(p, color);
                }
            }else{
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