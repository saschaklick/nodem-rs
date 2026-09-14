use log;
use core::fmt::Write;
use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::media::*;
use crate::media::font::*;

const PKG_HEADER: &str = "PKG0";

impl Media {
    fn rle_fixed_stream (source: &[u8]) -> Bytes {
        let mut res = BytesMut::new();
        res.put_u8(Encoding::RLEFixed as u8);
        
        let mut pos = 0usize;
        let mut last = (source[0] >> 7) & 1;
        let mut rpt_cnt = 0u8;
        let mut raw = 0;
        let mut raw_cnt = 0usize;
        loop {
            let bit = (source[pos / 8] >> (7 - (pos % 8))) & 1;            

            if last == bit && rpt_cnt < 63 {                
                rpt_cnt += 1;
            }else{
                if rpt_cnt >= 7 || rpt_cnt >= 63{
                    let byte = 1 << 7 | last << 6 | (rpt_cnt & 0b00111111);
                    res.put_u8(byte);                                        
                    raw = 0;
                    raw_cnt = 0;                    
                }
                rpt_cnt = 1;
                last = bit;                
            }            

            if raw_cnt < 7 {
                raw = (raw >> 1) | (bit << 6);
                raw_cnt += 1;
            }
            if raw_cnt == 7 && rpt_cnt < 7 {
                let byte = 0 << 7 | (raw & 0b01111111);
                res.put_u8(byte);                
                rpt_cnt = 0;
                raw_cnt = 0;
                raw = 0;
            }
            
            pos += 1;
            if pos / 8 >= source.len() {
                break;
            }
        }

        if raw_cnt != 0 && raw_cnt > rpt_cnt as usize {
            let byte = 0 << 7 | ((raw >> (7 - raw_cnt)) & 0b01111111);
            res.put_u8(byte);            
        }else if rpt_cnt > 0 {
            let byte = 1 << 7 | last << 6 | (rpt_cnt & 0b00111111);
            res.put_u8(byte);                                
        }

        Bytes::from(res)
    }
    
    fn optimize_stream (source: &[u8], use_rle: bool) -> Bytes{                    
        if source[0] != Encoding::Raw as u8 || use_rle == false{
            let mut res_raw = BytesMut::new();
            res_raw.put(source);
            return Bytes::from(res_raw);
        }        
        
        let res_rle = Media::rle_fixed_stream(&source[1..]);

        log::debug!("raw: {}", source.len());
        log::debug!("rle: {}", res_rle.len());                

        if source.len() <= res_rle.len()   {
            let mut res = BytesMut::new();
            res.put(source);              
            Bytes::from(res)
        }else{
           res_rle
        }
    }

    pub fn generate_pkg(self: &Media, sources: u8, optimize: bool) -> Bytes {
        let mut images_lib = BytesMut::new();
        let mut fonts_lib = BytesMut::new();
        let mut borders_lib = BytesMut::new();
        let mut pages_lib = BytesMut::new();
        let mut programs_lib = BytesMut::new();
        let mut end_lib = BytesMut::new();
        end_lib.put_u8(0xff);

        for index in 0 ..= 255 - 2 {
            let image = self.get_image(Identifier::Index(index));                 
            let image_data = &mut Bytes::copy_from_slice(image.data);                        
            let _type = image_data.get_u8();
            let width = image_data.get_u8();
            let height = image_data.get_u8();
            if (image.source >= LIBRARY_MAX) || width == 0 || height == 0 ||  ((1u8 << image.source) & sources) == 0 {
                continue;
            }                        
            let data = Media::optimize_stream(&image_data.slice(0 ..), optimize);
            let mask = if image.mask.is_some() { Media::optimize_stream(&image.mask.unwrap(), optimize) } else { Bytes::new() };            
            if data.len() >= u16::MAX as usize{
                log::error!("image data #{index} too large");
                continue;
            }
            if mask.len() >= u16::MAX as usize{
                log::error!("image mask #{index} too large");
                continue;
            }               
            Media::insert_image(&mut images_lib, index, image.id, width, height, data.to_vec().as_slice(), mask.to_vec().as_slice());                
            log::info!("img #{}.{} [{:5}b] [{}b/{}b]", image.source, index, data.len() + mask.len(), data.len(), mask.len());
        }
        images_lib.put_u8(0xff);

        for index in 0 ..= 255 - 2 {               
            let font = self.get_font(Identifier::Index(index));            
            if (font.source >= LIBRARY_MAX) || font.pack_count == 0 || ((1u8 << font.source) & sources) == 0 {
                continue;
            }
            let mut f = BytesMut::new();
            f.put_u8(font.pack_count);
            f.put_u8(font.mono_width);
            f.put_u8(font.base_height);
            f.put_u8(font.full_height);
            let mut packs = font.data;
            for _pack_index in 0 .. font.pack_count {                
                let start = packs.get_u8();
                let end = packs.get_u8();
                let length = packs.get_u16_ne();                
                let mut dir = BytesMut::new();
                let mut glyphs = BytesMut::new();                                                
                packs = &packs[length as usize ..];          
                for char in start .. end + 1 {
                    let glyph = self.get_glyph(&font, char as char);                    
                    if glyph.len() < GLYPH_HEADER_SIZE {                    
                        log::error!("pkg font #{}.{} glyph {} data too short (size: {}b)", font.source, index, char, glyph.len());
                    }else{
                        log::debug!("pkg font #{}.{} glpyh {} (size: {}b)", font.source, index, char, glyph.len());                    
                        let header = &glyph[0 .. GLYPH_HEADER_SIZE];
                        let data = &glyph[GLYPH_HEADER_SIZE .. glyph.len()];
                        glyphs.put(header.as_ref());
                        glyphs.put(Media::optimize_stream(data, true));                                                                    
                        dir.put_u16_ne(glyphs.len() as u16);                    
                    }
                }                
                f.put_u8(start);
                f.put_u8(end);
                f.put_u16_ne((dir.len() + glyphs.len()) as u16);
                f.put(dir.as_ref());
                f.put(glyphs.as_ref());
                log::debug!("fnt #{}.{} pack {} - {} (index: {}b data: {}b )", font.source, index, start, end, dir.len(), glyphs.len());
            }
            fonts_lib.put_u8(index);
            //fonts_lib.put_u8(0);
            fonts_lib.put_u8(font.id.len() as u8);
            fonts_lib.put(font.id.as_bytes());
            fonts_lib.put_u16_ne(f.len() as u16);
            fonts_lib.put(f.as_ref());

            log::info!("fnt #{}.{} [{:5}b]", font.source, index, f.len());            
        }
        fonts_lib.put_u8(0xff);

        #[cfg(feature = "ninepatch")]
        for index in 0 ..= 255 - 2 {
            let ninepatch = self.get_border(Identifier::Index(index));                 
            if (ninepatch.source >= LIBRARY_MAX) || (ninepatch.x_w == 0 && ninepatch.y_h == 0) || ((1u8 << ninepatch.source) & sources) == 0 {
                continue;
            }   
            let mut f = BytesMut::new();            
            f.put_u8(ninepatch.n_h);
            f.put_u8(ninepatch.e_w);
            f.put_u8(ninepatch.s_h);
            f.put_u8(ninepatch.w_w);
            f.put_u8(ninepatch.x_w);
            f.put_u8(ninepatch.y_h);
            for i in 0 .. 8 {
                f.put_u8(ninepatch.data[i].len() as u8);
            }
            for i in 0 .. 8 {
                f.put(ninepatch.data[i]);
            }
            if f.len() >= u16::MAX as usize{
                log::error!("patch data #{index} too large");
                continue;
            }
            borders_lib.put_u8(index);
            borders_lib.put_u8(ninepatch.id.len() as u8);
            borders_lib.put(ninepatch.id.as_bytes());
            borders_lib.put_u16_ne(f.len() as u16);            
            borders_lib.put(&mut f);

            log::info!("brd #{}.{} [{:5}b]", ninepatch.source, index, f.len());            
        }
        borders_lib.put_u8(0xff);

        #[cfg(feature = "dom")]
        for index in 0 ..= 255 - 2 {
            let page = self.get_page(index);                 
            if (page.source >= LIBRARY_MAX) || page.content.len() == 0 || ((1u8 << page.source) & sources) == 0 {
                continue;
            }                        
            Media::insert_page(&mut pages_lib, index, page.id, page.content);                
            log::info!("pag #{}.{} [{:5}b]", page.source, index, page.content.len());
        }
        pages_lib.put_u8(0xff);

        #[cfg(feature = "vm")]
        for index in 0 ..= 255 - 2 {
            let program = self.get_program(index);                 
            if (program.source >= LIBRARY_MAX) || program.data.len() == 0 || ((1u8 << program.source) & sources) == 0 {
                continue;
            }                        
            Media::insert_program(&mut programs_lib, index, program.id, program.data);                
            log::info!("pag #{}.{} [{:5}b]", program.source, index, program.data.len());
        }
        programs_lib.put_u8(0xff);
        
        let mut pkg = BytesMut::new();
        pkg.write_str(PKG_HEADER).unwrap();
        
        pkg.put_u32_ne(0);
        pkg.put_u16_ne(0);
        
        if images_lib.len() > 0 {
            pkg.put_u8(LIBMAGIC_IMAGE);
            pkg.put_u16_ne(images_lib.len() as u16);
            pkg.put(images_lib);
        }
        if borders_lib.len() > 0 {
            pkg.put_u8(LIBMAGIC_BORDER);
            pkg.put_u16_ne(borders_lib.len() as u16);
            pkg.put(borders_lib);
        }
        if fonts_lib.len() > 0 {
            pkg.put_u8(LIBMAGIC_FONT);
            pkg.put_u16_ne(fonts_lib.len() as u16);
            pkg.put(fonts_lib);
        }
        if pages_lib.len() > 0 {
            pkg.put_u8(LIBMAGIC_PAGE);
            pkg.put_u16_ne(pages_lib.len() as u16);
            pkg.put(pages_lib);
        }
        if programs_lib.len() > 0 {
            pkg.put_u8(LIBMAGIC_PROGRAM);
            pkg.put_u16_ne(programs_lib.len() as u16);
            pkg.put(programs_lib);
        }
        pkg.put(end_lib);

        let mut crc: u16 = 0x1002;
        for byte in &pkg[12..] {
            crc = crc.wrapping_add(*byte as u16);
        }

        let pkg_len = pkg.len();
        let mut pkg_start = &mut pkg[PKG_HEADER.len()..];
        pkg_start.put_u32_ne(pkg_len as u32);
        pkg_start.put_u16_ne(crc);                

        return pkg.into();
    }
}