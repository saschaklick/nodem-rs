use crate::media::*;

impl Media {
    pub fn inspect(&mut self, source: u8, csv: &mut dyn core::fmt::Write) -> core::fmt::Result {     
        let filter_source =  |src| -> bool {
            return (src == 255) && (source == 255 || source == src);
        };
        
        csv.write_str("image_idx,id,source,type,width,height\r\n").expect("");        
        for image_idx in 0 .. 255 {
            let image = self.get_image(Identifier::Index(image_idx));
            let data = image.data;                        
            if filter_source(image.source) || data[1] == 0 || data[2] == 0 { continue; }            
            csv.write_fmt(format_args!(
                "{},\"{}\",{},{},{},{}\r\n",
                image_idx, image.id, image.source, data[0], data[1], data[2]
            )).expect("");
        }
        
        #[cfg(feature = "ninepatch")]
        {
            csv.write_str("border_idx,id,source,n,e,s,w,h,v\r\n").expect("");         
            for border_idx in 0..BORDER_MAX {
                let border = self.get_border(Identifier::Index(border_idx));
                if filter_source(border.source) || border.data.map(|d| d.len()).iter().sum::<usize>() == 0 { continue; }
                csv.write_fmt(format_args!(
                    "{},\"{}\",{},{},{},{},{},{},{}\r\n",
                    border_idx, border.id, border.source,border.n_h, border.e_w, border.s_h, border.w_w, border.x_w, border.y_h
                )).expect("");
            }
            csv.write_fmt(format_args!("{},\"Solid\",0,1,1,1,1,1,1\r\n", BORDER_RECT)).expect("");         
        }
        
        csv.write_str("font_idx,id,source,height,base,mono\r\n").expect("");
        for font_idx in 0..FONT_MAX {
            let font = self.get_font(Identifier::Index(font_idx));
            if filter_source(font.source) || font.pack_count == 0 { continue; }
            csv.write_fmt(format_args!(
                "{},\"{}\",{},{},{},{}\r\n",
                font_idx, font.id, font.source, font.base_height, font.full_height, font.mono_width
            )).expect("");
        }        

        csv.write_str("page_idx,id,source,size\r\n").expect("");
        for page_idx in 0..255 {
            let page = self.get_page(page_idx);
            if filter_source(page.source) || page.content.len() == 0 { continue; }
            csv.write_fmt(format_args!(
                "{},\"{}\",{},{}\r\n",
                page_idx, page.id, page.source, page.content.len()
            )).expect("");
        }     

        #[cfg(feature = "vm")]
        {
            csv.write_str("program_idx,id,source,size\r\n").expect("");
            for program_idx in 0..255 {
                let program = self.get_program(program_idx);
                if filter_source(program.source) || program.data.len() == 0 { continue; }
                csv.write_fmt(format_args!(
                    "{},\"{}\",{},{}\r\n",
                    program_idx, program.id, program.source, program.data.len()
                )).expect("");
            }     
        }

        csv.write_str("@0\r\n").expect("");

        Ok(())
    }
}