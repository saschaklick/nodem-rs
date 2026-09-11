use crate::dom::*;
use crate::surface::*;

use core::cmp;

impl Surface {    
    pub fn update(self: &mut Self, dom: &mut DOM) {        
        let mut result = NodePosition::new();

        self.media.drawing = media::image::Settings::default();
        self.media.typesetting = media::font::Settings::default();
        
        dom.calculate(self, Size { width: self.width, height: self.height }, &mut result);
        
        self.render(&dom, &result);
                                  
        Surface::update_positions(dom, result);
    }

    #[cfg(feature = "inspect")]
    fn update_positions(dom: &mut DOM, positions: NodePosition) { dom.positions = positions; }
    #[cfg(not(feature = "inspect"))]
    fn update_positions(_dom: &mut DOM, _positions: NodePosition) {}

    pub fn fullscreen(&self, dom: &mut DOM) {
        if NODE_MAX > 0 {
            dom.nodes[0].plot.size.width = WIDTH_FLEX;
            dom.nodes[0].plot.size.height = HEIGHT_FLEX;
        }
    }

    fn render(&mut self, dom: &DOM, positionings: &NodePosition) {
        let mut p_stack: [NodeIdx; DOM_DEPTH_MAX] = [NODE_MAX; DOM_DEPTH_MAX];
        let mut p_clips: [Clip; DOM_DEPTH_MAX] = [Clip { p0: Point { x: 0, y: 0 }, p1: Point { x: self.width as PosX, y: self.height as PosY } }; DOM_DEPTH_MAX];
        let p_typesetting = [media::font::Settings::default(); DOM_DEPTH_MAX];
        
        let mut n_i: NodeIdx = 0;
        let mut p_i: usize = 0;
        let mut descent: u8 = 0;
        
        //log::debug!("");
        //log::debug!("{}", dom);

        self.media.typesetting.active_idx = self.media.typesetting.default_idx;

        loop {        
            let node = &dom.nodes[n_i as usize];

            self.media.typesetting = p_typesetting[p_i];            

            if descent == 0 {
                if node.visibility() == Visibility::Hidden {
                    descent = 1;
                }else{
                    let position = &positionings.areas[n_i as usize]; 
                    let mut p = position.0.point;        
                    let mut s  = position.0.size;
                    let c      = position.1;

                    let margin = dom.margins[node.plot.margin_idx as usize];                    
                    let style = dom.styles[node.plot.style_idx as usize];
                                        
                    log::trace!(" #{n_i:02} {} => [{:3}, {:3}] ({:3}, {:3})", node, s.width, s.height, p.x, p.y);                    

                    p.x += margin.w as PosX;
                    p.y += margin.n as PosY;
                    s.width = s.width.checked_sub((margin.e + margin.w) as SizeW).unwrap_or(0);
                    s.height = s.height.checked_sub((margin.n + margin.s) as SizeH).unwrap_or(0);                                        
                    
                    let clip_p = p_clips[p_i];                    
                    let clip_n = Clip {
                        p0 : Point {
                            x : cmp::max( clip_p.p0.x, p.x),
                            y : cmp::max( clip_p.p0.y, p.y)
                        },
                        p1 : Point {
                            x : cmp::min( clip_p.p1.x, p.x.checked_add(s.width as PosX).unwrap_or(PosX::MAX)),
                            y : cmp::min( clip_p.p1.y, p.y.checked_add(s.height as PosY).unwrap_or(PosY::MAX))
                        }
                    };
                    if p_i < DOM_DEPTH_MAX as usize - 1 {                        
                        let pad = dom.paddings[node.plot.padding_idx as usize];
                        
                        p_clips[p_i + 1] = Clip {
                            p0 : Point {
                                x : clip_n.p0.x + pad.w as PosX,
                                y : clip_n.p0.y + pad.n as PosY,
                            },
                            p1 : Point {
                                x : clip_n.p1.x - pad.e as PosX,
                                y : clip_n.p1.y - pad.s as PosY,
                            }
                        };
                    };                    
                    let clip_p = p_clips[p_i + 1];                   
                    self.clip = clip_n;

                    let hidden= node.visibility() == Visibility::Hidden;

                    if !hidden && style.background != Color::MAX {
                        self.fill_rect(Area { point: p, size : s }, self.palette[style.background as usize]);
                    }                                        
                                    
                    self.media.typesetting.inverted = style.color == 0;
                    if style.font_idx < FONT_MAX {
                        self.media.typesetting.active_idx = style.font_idx;
                    }
                    
                    if !hidden && node.plot.content_idx != CONTENT_MAX {
                        let content = dom.contents[node.plot.content_idx as usize];
                        if content.as_str().len() > 0 {
                            let pad = dom.full_padding(*node, self);
                            const V_ALIGN: u8 = 1;
                            const H_ALIGN: u8 = 1;
                            let position = Point {
                                // x: p.x + pad.w as PosX,
                                // y: p.y + pad.n as PosY,
                                x: p.x.saturating_add(pad.w as PosX).saturating_add(if V_ALIGN == 0 { 0 } else { core::cmp::max(0, (s.width.saturating_sub(c.width) as PosX).saturating_sub(pad.w.saturating_add(pad.e) as PosX)) / if V_ALIGN != 2 { 2 } else { 1 } }),
                                y: p.y.saturating_add(pad.n as PosY).saturating_add(if H_ALIGN == 0 { 0 } else { core::cmp::max(0, (s.height.saturating_sub(c.height) as PosY).saturating_sub(pad.n.saturating_add(pad.s) as PosY)) / if H_ALIGN != 2 { 2 } else { 1 } })                               
                            };
                            self.clip = clip_p;
                            content.draw(self, position);                            
                            self.clip = clip_n;
                        }
                    }                     
                                        
                    if !hidden {
                        self.draw_border(style.border_idx, Area{ point: p, size: s });                    
                    }
                    
                    if node.first_child == NODE_MAX {
                        descent = 1;                                              
                    }else{     
                        p_stack[p_i] = n_i;
                        p_i += 1;
                        n_i = node.first_child;
                    }

                    //log::info!("");
                }
            }else if descent == 1 {
                if p_i == 0 {
                    break;
                }                

                let style = dom.styles[node.plot.style_idx as usize];
                if style.font_idx < FONT_MAX {
                    self.media.typesetting.active_idx = style.font_idx;
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

        self.clip.reset(self.width, self.height);
    }
}