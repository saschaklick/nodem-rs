use log;
use core::fmt::Write;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use nostd::vec::Vec;

use crate::media::{ *, font::*, image::* };

const PKG_HEADER: &str = "PKG0";

impl Media {
    fn index1_to_rle (source: &[u8]) -> Bytes {
        let mut res = BytesMut::new();
        res.put_u8(Encoding::Index1RLEFixed as u8);
        
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
    
    fn rgb888_to_rle (source: &[u8]) -> Bytes {
        let count = source.len() / 3;
        let pixels = |i: usize| &source[i * 3..i * 3 + 3];
        let mut res = BytesMut::new();
        res.put_u8(Encoding::Rgb888RLEFixed as u8);
        
        let run_at = |i: usize| -> usize {
            let mut n = 1;
            while n < 128 && i + n < count && pixels(i + n) == pixels(i) {
                n += 1;
            }
            n
        };

        let mut i = 0usize;
        while i < count {
            let run = run_at(i);
            if run >= 2 {
                res.put_u8(0x80 | (run - 1) as u8);
                res.put(pixels(i));
                i += run;
            } else {                
                let start = i;
                while i < count && i - start < 128 && run_at(i) < 2 {
                    i += 1;
                }
                res.put_u8((i - start - 1) as u8);
                res.put(&source[start * 3..i * 3]);
            }
        }

        Bytes::from(res)
    }

    fn rgb888_to_index8 (source: &[u8]) -> Option<Bytes> {        
        let mut res = BytesMut::new();
        res.put_u8(Encoding::Index8Raw as u8);
        
        let pixels = |i: usize| &source[i * 3..i * 3 + 3];
        let palette: Vec<_> = (0..source.len() / 3).filter(|&i| !(0..i).any(|j| pixels(j) == pixels(i))).map(|i| &source[i * 3..i * 3 + 3]).collect();   

        if palette.len() > u8::MAX as usize {
            return None;
        }

        res.put_u8(palette.len() as u8);
        for color in &palette {
            res.put_slice(color);
        }        
                
        for i in 0..source.len() / 3 {            
            res.put_u8(palette.iter().position(|n| *n == pixels(i)).unwrap_or(255) as u8);
        }        

        Some(Bytes::from(res))
    }
    
    fn optimize_stream (source: &[u8], use_rle: bool) -> Bytes{                      
        if use_rle {
            if source[0] == Encoding::Index1Raw as u8 {
                let res_rle = Media::index1_to_rle(&source[1..]);
                if source.len() > res_rle.len()   {              
                    log::debug!("index1 - raw: {:5} rle: {:5} rate: {:3.1}%", source.len(), res_rle.len(), res_rle.len() as f32 / source.len() as f32 * 100.0);                    
                    return res_rle;
                }
            }else
            if source[0] == Encoding::Rgb888Raw as u8 {                
                let rle = Media::rgb888_to_rle(&source[1..]);                
                let in8 = Media::rgb888_to_index8(&source[1..]);                                               
                
                let in8_len = in8.as_ref().map_or(source.len() * 2, |n| n.len());

                log::info!("rgb888 - raw: {:5} rle: {:5} ({:3.1}%) in8: {:5} ({:3.1}%)", source.len(), rle.len(), rle.len() as f32 / source.len() as f32 * 100.0, in8_len, in8_len as f32 / source.len() as f32 * 100.0);                    

                if in8_len < rle.len() && in8_len < source.len() {
                    return in8.unwrap();
                }else
                if rle.len() < source.len() {
                    return rle;
                }
            }
        }
        let mut res = BytesMut::new();
        res.put(source);   
        Bytes::from(res)                            
    }

    pub fn generate_pkg(self: &Media, sources: u8, meta: &str, optimize: bool) -> Bytes {        
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
            let format = image_data.get_u8();
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
            Media::insert_image(&mut images_lib, index, image.id, format, width, height, data.to_vec().as_slice(), mask.to_vec().as_slice());                
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
        
        if meta.len() > 0 {
            pkg.put_u8(LIBMAGIC_META);    
            pkg.put_u16_ne(meta.len() as u16);
            pkg.put(meta.as_bytes());
        }
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

#[cfg(test)]
mod rle_tests {
    extern crate std;
    use std::{vec, vec::Vec};
    use super::*;

    fn decode(rle: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut i = 1; // skip encoding byte
        while i < rle.len() {
            let h = rle[i]; i += 1;
            let n = (h & 0x7f) as usize + 1;
            if h & 0x80 != 0 {
                for _ in 0..n { out.extend_from_slice(&rle[i..i + 3]); }
                i += 3;
            } else {
                out.extend_from_slice(&rle[i..i + n * 3]);
                i += n * 3;
            }
        }
        out
    }

    fn check(src: Vec<u8>) {
        let rle = Media::rgb888_to_rle(&src);
        assert_eq!(decode(&rle), src);
    }

    #[test]
    fn roundtrip() {
        check(vec![]);
        check(vec![1, 2, 3]);
        check(vec![1, 2, 3, 4, 5, 6]);
        check([9, 9, 9].repeat(500));
        check([1, 2, 3, 1, 2, 3, 7, 7, 7].repeat(100));
        let mixed: Vec<u8> = (0..3000u32).map(|i| ((i / 3 * 7919) % 5) as u8 * ((i % 3) as u8 + 1)).collect();
        check(mixed);
        let rle = Media::rgb888_to_rle(&[5, 5, 5].repeat(130));
        assert_eq!(&rle[1..], &[0x80 | 127, 5, 5, 5, 0x80 | 1, 5, 5, 5]);
    }
}
