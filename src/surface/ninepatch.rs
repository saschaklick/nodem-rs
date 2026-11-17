use crate::stream::*;
use crate::surface::*;

#[cfg(feature = "ninepatch")]
use core::cmp;

pub use u8 as NinePatchW;
pub use u8 as NinePatchH;

pub struct NinePatch <'a> {
   pub source: u8,
   pub id:  &'a str,
   pub n_h: NinePatchH,
   pub e_w: NinePatchW,   
   pub s_h: NinePatchH,
   pub w_w: NinePatchW,
   pub x_w: NinePatchW,
   pub y_h: NinePatchW,   
   pub data: [&'a [u8]; 8]
}
impl Default for NinePatch<'_> {
   fn default() -> Self {
      Self {
         source: 255,
         id : "",
         n_h: 1,
         e_w: 1,
         s_h: 1,
         w_w: 1,
         x_w: 2,
         y_h: 2,  
         data: [       
            &[Encoding::Index1Raw as u8, 0b01000000],
            &[Encoding::Index1Raw as u8, 0b10000000],
            &[Encoding::Index1Raw as u8, 0b10000000],
            &[Encoding::Index1Raw as u8, 0b10000000],
            &[Encoding::Index1Raw as u8, 0b10000000],
            &[Encoding::Index1Raw as u8, 0b10000000],
            &[Encoding::Index1Raw as u8, 0b01000000],
            &[Encoding::Index1Raw as u8, 0b10000000]
         ]
      }
   }
}

impl Surface {
   pub fn draw_border(&self, border_idx: BorderIdx, target: Area, colors: Option<&[ColorType]>) {
      let colors = colors.unwrap_or(&self.palette);
      #[cfg(not(feature = "ninepatch"))]
      if border_idx != BORDER_MAX {
         self.draw_rect(target, colors[1]);
      }
      
      #[cfg(feature = "ninepatch")]
      if border_idx == BORDER_RECT {
         self.draw_rect(target, colors[1]);
      }else
      if border_idx < BORDER_RECT {
         self.draw_ninepatch(Identifier::Index(border_idx), target, colors);
      }
   }
   
   #[cfg(feature = "ninepatch")]
   pub fn draw_ninepatch(&self, ident: Identifier, target: Area, colors: &[ColorType]) {
      let ninepatch = self.media.get_border(ident);

      let nw_outer = target.point;
      let nw_inner = Point {
         x: cmp::min(target.point.x.saturating_add(target.size.width as PosX), target.point.x.saturating_add(ninepatch.w_w as PosX)),
         y: cmp::min(target.point.y.saturating_add(target.size.height as PosY), target.point.y.saturating_add(ninepatch.n_h as PosY))
      };
      let se_outer = Point {
         x: target.point.x.saturating_add(target.size.width as PosX),
         y: target.point.y.saturating_add(target.size.height as PosY)
      };
      let se_inner = Point {
         x: target.point.x.saturating_add(target.size.width as PosX).saturating_sub(ninepatch.e_w as PosX),
         y: target.point.y.saturating_add(target.size.height as PosY).saturating_sub(ninepatch.s_h as PosY)
      };
      
      if ninepatch.x_w > 0 { // x
         let mut pos = Point { x : nw_inner.x, y: 0 };            
         let mut reader_n = PixelStream::new(ninepatch.data[0], &colors);  
         let mut reader_s = PixelStream::new(ninepatch.data[4], &colors);  
         for p_x in 0 .. se_inner.x.saturating_sub(nw_inner.x) {      
            pos.y = nw_outer.y;                     
            for _p_y in 0 .. ninepatch.n_h {               
               self.draw_pixel(pos, reader_n.next());
               pos.y = pos.y.saturating_add(1);            
            }
            pos.y = se_outer.y - 1;
            for _p_y in 0 .. ninepatch.s_h {               
               self.draw_pixel(pos, reader_s.next());
               pos.y = pos.y.saturating_sub(1);            
            }
            if p_x % ninepatch.x_w as PosX == (ninepatch.x_w as PosX).saturating_sub(1) {
               reader_n.reset();
               reader_s.reset();
            }
            pos.x = pos.x.saturating_add(1);
         }      
      }   

      if ninepatch.y_h > 0 { // y
         let mut pos = Point { x : 0, y: nw_inner.y };            
         let mut reader_e = PixelStream::new(ninepatch.data[2], colors);  
         let mut reader_w = PixelStream::new(ninepatch.data[6], colors);  
         for p_y in 0 .. se_inner.y.saturating_sub(nw_inner.y) {      
            pos.x = nw_outer.x;
            for _p_x in 0 .. ninepatch.w_w {               
               self.draw_pixel(pos, reader_w.next());
               pos.x = pos.x.saturating_add(1);            
            }
            pos.x = se_outer.x - 1;
            for _p_x in 0 .. ninepatch.e_w {               
               self.draw_pixel(pos, reader_e.next());
               pos.x = pos.x.saturating_sub(1);            
            }
            if p_y % ninepatch.y_h as PosX == (ninepatch.y_h as PosX).saturating_sub(1) {
               reader_e.reset();
               reader_w.reset();
            }
            pos.y = pos.y.saturating_add(1);
         }      
      }  

      struct Edge {
         point  : Point,
         patch  : usize,
         width  : u8,
         height : u8
      }
      let edges = [
         Edge { point: Point { x: nw_outer.x, y: nw_outer.y }, patch: 7, width: ninepatch.w_w, height: ninepatch.n_h },
         Edge { point: Point { x: se_outer.x.saturating_sub(1), y: nw_outer.y }, patch: 1, width: ninepatch.e_w, height: ninepatch.n_h },
         Edge { point: Point { x: nw_outer.x , y: se_outer.y.saturating_sub(1) }, patch: 5, width: ninepatch.w_w, height: ninepatch.s_h },
         Edge { point: Point { x: se_outer.x.saturating_sub(1), y: se_outer.y.saturating_sub(1) }, patch: 3, width: ninepatch.e_w, height: ninepatch.s_h },        
      ];
      for (i, edge) in edges.iter().enumerate() {         
         let mut pos = edge.point;
         let mut reader = PixelStream::new(ninepatch.data[edge.patch], &colors);  
         for _p_y in 0 .. edge.height {         
            pos.x = edge.point.x;
            for _p_x in 0 .. edge.width {               
               self.draw_pixel(pos, reader.next());
               pos.x = if (i & 1) == 0 { pos.x.saturating_add(1) } else { pos.x.saturating_sub(1) };
            }
            pos.y = if (i & 2) == 0 { pos.y.saturating_add(1) } else { pos.y.saturating_sub(1) };
         }      
      }                  
   }
}