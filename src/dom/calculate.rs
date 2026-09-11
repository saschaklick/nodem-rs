use crate::node::{ Alignment };
use crate::dom::*;
use crate::surface::*;
use crate::media::font::Settings;

impl DOM {
    pub(crate) fn full_padding (&self, node: Node, surface: &Surface) -> Padding {
        let p = self.paddings[node.plot.padding_idx as usize];                        
        if node.framing() {
            let border_idx = self.styles[node.plot.style_idx as usize].border_idx;
            if border_idx < BORDER_MAX || border_idx == 254 {
                let b = surface.media.get_border(media::Identifier::Index(border_idx));
                Padding { n: p.n.saturating_add(b.n_h as i8) + 1, e: p.e.saturating_add(b.e_w as i8) + 1, s: p.s.saturating_add(b.s_h as i8) + 1, w: p.w.saturating_add(b.w_w as i8) + 1 }
            }else{
                p
            }
        }else{
            p
        }
    }
    
    pub fn calculate(&self, surface: &mut Surface, root_size: Size, result: &mut NodePosition) {        
        let mut p_stack: [NodeIdx; DOM_DEPTH_MAX] = [NODE_MAX; DOM_DEPTH_MAX];
        let p_typesetting = [Settings::default(); DOM_DEPTH_MAX];
        
        // Determine size of each node according to its content, i.e. children.
        let mut n_i: NodeIdx = 0;
        let mut p_i: usize = 0;
        let mut descent: u8 = 0;

        if NODE_MAX == 0 {
            return;
        }

        loop {        
            let node = self.nodes[n_i as usize];                        
            let pad_n = self.full_padding(node, surface); //self.paddings[node.plot.padding_idx as usize];
            let mar_n = self.margins[node.plot.margin_idx as usize];

            surface.media.typesetting = p_typesetting[p_i];            

            //log::debug!("1) dir: {:1} n_i: {:3} p_i {:3} | {:3} {:3} | {:3} {:3} | {:02x}", descent, n_i, p_ii, res_n.0.point.x, res_n.0.point.y, res_n.0.size.width, res_n.0.size.height, node.flags); 
            
            if descent == 0 {
                let style = self.styles[node.plot.style_idx as usize];
                if style.font_idx < FONT_MAX {
                    surface.media.typesetting.active_idx = style.font_idx;
                }         

                let res_c = &mut result.areas[n_i as usize].1;                
                let mut size_n = Size { width : 0, height : 0 };                
                if node.plot.content_idx != CONTENT_MAX {
                    let content = self.contents[node.plot.content_idx as usize];
                    if content.as_str().len() > 0 {                    
                        let content_size = content.get_size(&surface);                                                      
                        size_n.width = content_size.width;
                        size_n.height = content_size.height;                        
                        res_c.width = content_size.width;
                        res_c.height = content_size.height;
                    }else{
                        res_c.clear();
                    }
                }else{
                    res_c.clear();
                }
                let res_s = &mut result.areas[n_i as usize].0.size;
                if node.plot.size.width >= WIDTH_MAX {
                    size_n.width = size_n.width.saturating_add(mar_n.e.saturating_add(mar_n.w) as SizeW);
                    size_n.width = size_n.width.saturating_add(pad_n.e.saturating_add(pad_n.w) as SizeW);                
                    res_s.width = size_n.width;           
                }else{
                    res_s.width = node.plot.size.width.saturating_add(mar_n.e.saturating_add(mar_n.w) as SizeH);           
                }
                if node.plot.size.height >= HEIGHT_MAX {
                    size_n.height = size_n.height.saturating_add(mar_n.n.saturating_add(mar_n.s) as SizeH);
                    size_n.height = size_n.height.saturating_add(pad_n.n.saturating_add(pad_n.s) as SizeH);                
                    res_s.height = size_n.height;  
                }else{
                    res_s.height = node.plot.size.height.saturating_add(mar_n.n.saturating_add(mar_n.s) as SizeH);
                }            

                if node.first_child == NODE_MAX {
                    descent = 1;                        
                }else{                
                    p_stack[p_i] = n_i;
                    p_i += 1;
                    n_i = node.first_child;
                }
            }else if descent == 1 {
                if p_i == 0 {
                    break;
                }
                
                let parent = self.nodes[p_stack[p_i - 1] as usize];            
                let mar_p = self.margins[parent.plot.margin_idx as usize];            
                let pad_p = self.full_padding(parent, surface); //self.paddings[parent.plot.padding_idx as usize];            
                let dir_p = if ((self.nodes[p_stack[p_i - 1] as usize].flags >> FLAGS_DIRECTION_SHIFT) & FLAGS_DIRECTION_MASK) == 0 { Direction::Horizontal } else { Direction::Vertical };

                let res_n = result.areas[n_i as usize];            
                let res_p = &mut result.areas[p_stack[p_i - 1] as usize];

                let style = self.styles[node.plot.style_idx as usize];
                if style.font_idx < FONT_MAX {
                    surface.media.typesetting.active_idx = style.font_idx;
                }               

                if parent.plot.size.width >= WIDTH_MAX || parent.plot.size.height >= HEIGHT_MAX {
                    let mut size_p = Size { width: 0, height: 0 };
                    
                    size_p.width = size_p.width.checked_add(mar_p.e as SizeW + mar_p.w as SizeW + pad_p.e as SizeW + pad_p.w as SizeW).unwrap_or(SizeW::MAX);
                    size_p.height = size_p.height.checked_add(mar_p.n as SizeH + mar_p.s as SizeH + pad_p.n as SizeH + pad_p.s as SizeH).unwrap_or(SizeH::MAX);
                    let res_s = &mut res_p.0.size;
                    if dir_p == Direction::Vertical {                    
                        if parent.plot.size.width >= WIDTH_MAX {                        
                            res_s.width = core::cmp::max(res_s.width, size_p.width.checked_add(res_n.0.size.width).unwrap_or(SizeW::MAX));
                        }
                        if parent.plot.size.height >= HEIGHT_MAX {                        
                            res_s.height = res_s.height.checked_add(res_n.0.size.height).unwrap_or(SizeH::MAX);                    
                        }
                    }else{
                        if parent.plot.size.width >= WIDTH_MAX {                        
                            res_s.width = res_s.width.checked_add(res_n.0.size.width).unwrap_or(SizeW::MAX);
                        }
                        if parent.plot.size.height >= HEIGHT_MAX {                        
                            res_s.height = core::cmp::max(res_s.height, size_p.height.checked_add(res_n.0.size.height).unwrap_or(SizeH::MAX));
                        }
                    }
                }            
                
                if n_i > 0 && node.next_sibling != NODE_MAX {
                    descent = 0;
                    n_i = node.next_sibling;
                }else{
                    if p_i > 0 {
                        p_i -= 1;
                        n_i = p_stack[p_i];               
                    }else{
                        break;
                    }
                }
            }
        }        

        if self.nodes[0].plot.size.width == WIDTH_FLEX {
            result.areas[0].0.size.width = root_size.width;
        }
        if self.nodes[0].plot.size.height == HEIGHT_FLEX {
            result.areas[0].0.size.height = root_size.height;
        }

        // Place children inside parents.
        n_i = 0;
        p_i = 0;
        let mut l_stack: [NodeIdx; DOM_DEPTH_MAX] = [NODE_MAX; DOM_DEPTH_MAX];    
        descent = 0;    
        
        loop {        
            let node = self.nodes[n_i as usize];                

            //log::info!("> {} n:{} p:{} l:{}", descent, n_i, if p_i > 0 { p_stack[p_i - 1]} else { NODE_MAX }, if p_i > 0 { l_stack[p_i] } else { NODE_MAX });

            if descent == 0 {

                if p_i > 0 {
                    let parent = self.node_ref(p_stack[p_i - 1]);  
                    let mar_p = self.margins[parent.plot.margin_idx as usize];
                    let pad_p = self.full_padding(*parent, surface); //self.paddings[parent.plot.padding_idx as usize];          
                    let res_p = result.areas[p_stack[p_i - 1] as usize];  
                    let dir_p = parent.direction();                
                    
                    let res_n = &mut result.areas[n_i as usize];              
                    
                    if dir_p == Direction::Horizontal {
                        if node.alignment() == Alignment::Stretch {
                            res_n.0.size.height = res_p.0.size.height.checked_sub(mar_p.n as SizeH + mar_p.s as SizeH + pad_p.n as SizeH + pad_p.s as SizeH).unwrap_or(SizeH::MIN);
                        }else
                        if node.alignment() == Alignment::Center || node.alignment() == Alignment::End {
                            let mut space = res_p.0.size.height.checked_sub(res_n.0.size.height).unwrap_or(0);                
                            space = space.checked_sub(pad_p.n as SizeH + pad_p.s as SizeH + mar_p.n as SizeH + mar_p.s as SizeH).unwrap_or(SizeH::MIN);                
                            if node.alignment() == Alignment::Center {
                                space = space / 2;
                            }                                             
                            res_n.0.point.y += space as PosY;
                        }                
                    }else{
                        if node.alignment() == Alignment::Stretch {
                            res_n.0.size.width = res_p.0.size.width.checked_sub(mar_p.e as SizeW + mar_p.w as SizeW + pad_p.e as SizeW + pad_p.w as SizeW).unwrap_or(SizeW::MIN);
                        }else
                        if node.alignment() == Alignment::Center || node.alignment() == Alignment::End {
                            let mut space = res_p.0.size.width.checked_sub(res_n.0.size.width).unwrap_or(0);                
                            space = space.checked_sub(pad_p.e as SizeW + pad_p.w as SizeW + mar_p.e as SizeW + mar_p.w as SizeW).unwrap_or(SizeW::MIN);                
                            if node.alignment() == Alignment::Center {
                                space = space / 2;
                            }                                             
                            res_n.0.point.x += space as PosX;
                        }                
                    }
                }

                if node.first_child != NODE_MAX {
                    let res_n = result.areas[n_i as usize]; 
                    let dir_n = node.direction();
                    let mar_n = self.margins[node.plot.margin_idx as usize];
                    let pad_n = self.full_padding(node, surface); //self.paddings[node.plot.padding_idx as usize];
                    
                    let mut c_i = node.first_child;
                    let mut fixed_size: u16 = 0;
                    let mut flex_cnt: u16 = 0;
                    let mut flex_margin: i16 = 0;
                    loop {
                        let child = self.nodes[c_i as usize];
                        let mar_c = self.margins[child.plot.margin_idx as usize];
                        let res_c = &mut result.areas[c_i as usize];
                        let res_s = &mut res_c.0.size;
                        let flex_w_c = child.plot.size.width > WIDTH_MAX && child.plot.size.width <= WIDTH_FLEX;
                        let flex_h_c = child.plot.size.height > HEIGHT_MAX && child.plot.size.height <= HEIGHT_FLEX;
                        if dir_n == Direction::Horizontal {
                            if flex_w_c {
                                let flex_c = (FLEX_MAX + 1) as u16 - (child.plot.size.width - WIDTH_MAX) as u16;
                                flex_cnt += flex_c;               
                            }else{
                                fixed_size += res_s.width as u16;
                            }
                            if flex_h_c {
                                res_s.height = res_n.0.size.height.saturating_sub(pad_n.n as SizeH + pad_n.s as SizeH);
                                //res_s.height = res_s.height.saturating_sub(mar_c.n as SizeH + mar_c.s as SizeH);
                                flex_margin += (mar_c.e + mar_c.w) as i16;                              
                            }
                        }else{
                            if flex_w_c {
                                res_s.width = res_n.0.size.width.saturating_sub(pad_n.e as SizeW + pad_n.w as SizeW);
                                //res_s.width = res_s.width.saturating_sub(mar_c.e as SizeW + mar_c.w as SizeW);                
                            }
                            if flex_h_c {
                                let flex_c = (FLEX_MAX + 1) as u16 - (child.plot.size.height - HEIGHT_MAX) as u16;
                                flex_cnt += flex_c;
                                flex_margin += (mar_c.n + mar_c.s) as i16;                              
                            }else{
                                fixed_size += res_s.height as u16;
                            }
                        }
                        if child.next_sibling != NODE_MAX {
                            c_i = child.next_sibling;
                        }else{
                            break;
                        }
                    }
                    
                    if flex_cnt > 0 {
                        c_i = node.first_child;
                        let mut flex_size: u16 = if dir_n == Direction::Horizontal {
                            res_n.0.size.width.saturating_sub(pad_n.w as SizeW + pad_n.e as SizeW + mar_n.w as SizeW + mar_n.e as SizeW) as u16
                        } else {
                            res_n.0.size.height.saturating_sub(pad_n.n as SizeH + pad_n.s as SizeH + mar_n.n as SizeH + mar_n.s as SizeH) as u16
                        };
                        flex_size = flex_size.saturating_sub(fixed_size).saturating_sub(flex_margin as u16);
                        let flex_step = flex_size.saturating_div(flex_cnt);
                        let mut flex_remain = flex_size - (flex_step * flex_cnt);                    
                        //log::info!("#{:02} flex: {} {} {}", n_i, flex_cnt, flex_size, flex_step);
                        loop {
                            let child = self.nodes[c_i as usize];
                            let mar_c = self.margins[child.plot.margin_idx as usize];
                            let res_c = &mut result.areas[c_i as usize];
                            let res_s = &mut res_c.0.size;
                            if dir_n == Direction::Horizontal {
                                let flex_w_c = child.plot.size.width > WIDTH_MAX && child.plot.size.width <= WIDTH_FLEX;
                                if flex_w_c {
                                    let flex_c = (FLEX_MAX + 1) as u16 - (child.plot.size.width - WIDTH_MAX) as u16;
                                    res_s.width = (flex_step as SizeW).saturating_mul(flex_c as SizeW).saturating_add(flex_remain).saturating_add(mar_c.e.saturating_add(mar_c.w) as SizeW);
                                    flex_remain = flex_remain.saturating_sub(1);
                                }
                            }else{
                                let flex_h_c = child.plot.size.height > HEIGHT_MAX && child.plot.size.height <= HEIGHT_FLEX;
                                if flex_h_c {
                                    let flex_c = (FLEX_MAX + 1) as u16 - (child.plot.size.height - HEIGHT_MAX) as u16;
                                    res_s.height = (flex_step as SizeH).saturating_mul(flex_c as SizeH).saturating_add(flex_remain).saturating_add(mar_c.n.saturating_add(mar_c.s) as SizeH);
                                    flex_remain = flex_remain.saturating_sub(1);
                                }
                            }
                            if child.next_sibling != NODE_MAX {
                                c_i = child.next_sibling;
                            }else{
                                break;
                            }
                        } 
                    }

                    c_i = node.first_child;
                    let mut pos_n = Point { x: res_n.0.point.x + pad_n.w as PosX, y: res_n.0.point.y + pad_n.n as PosY };
                    loop {
                        let child = self.nodes[c_i as usize];
                        let res_c = &mut result.areas[c_i as usize];
                        let res_s = &mut res_c.0.size;
                        res_c.0.point.x = pos_n.x.saturating_add(mar_n.e as PosX);
                        res_c.0.point.y = pos_n.y.saturating_add(mar_n.n as PosY);
                        //log::info!("#{:02} #{:02}: x: {} y: {} w: {} h: {}", n_i, c_i, res_c.point.x, res_c.point.y, res_s.width, res_s.height);
                        if dir_n == Direction::Horizontal {
                            pos_n.x = pos_n.x.saturating_add(res_s.width as PosX);
                        }else{                        
                            pos_n.y = pos_n.y.saturating_add(res_s.height as PosY);
                        }
                        if child.next_sibling != NODE_MAX {
                            c_i = child.next_sibling;
                        }else{
                            break;
                        }
                    }                       
                }     

                if node.first_child == NODE_MAX {
                    descent = 1;                
                }else{                
                    p_stack[p_i] = n_i;
                    l_stack[p_i] = NODE_MAX;
                    p_i += 1;
                    n_i = node.first_child;                
                }
            }else if descent == 1 {
                if p_i == 0 {
                    break;
                }            
                
                if node.next_sibling != NODE_MAX {
                    descent = 0;
                    l_stack[p_i] = n_i;
                    n_i = node.next_sibling;
                }else{
                    if p_i > 0 {                    
                        p_i -= 1;                    
                        n_i = p_stack[p_i];               
                    }else{
                        break;
                    }
                }
            }    
        }  
        
    }

}