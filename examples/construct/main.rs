use cfg_block::cfg_block;

cfg_block! {
    if #[cfg(feature = "ui")] {
        #[path = "../../impl/sdl2.rs"]
        mod sdl2;
        use sdl2::Example;
    } else {
        #[path = "../../impl/term.rs"]        
        mod term;
        use term::Example;
    }
}
    
use nodem_rs::*;
use nodem_rs::node::*;
use nodem_rs::media::Identifier;
use nodem_rs::surface::*;

pub static PKG_SYS: &'static [u8] = include_bytes!("../../../nodem-pkg/pkg/sys.pkg");
pub static PKG_INT: &'static [u8] = include_bytes!("../../../nodem-pkg/pkg/int.pkg");

impl Example {
    pub fn init(self: &mut Self, surface: &mut Surface) {
        surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
        surface.media.load_pkg(PKG_INT.as_ptr(), PKG_INT.len(), 1);
    }

    pub fn repeat(self: &mut Self, surface: &mut Surface, repeat_cnt: u32, cursor: Point, _action: u8) {
        let dom = &mut self.dom;
        
        dom.set_child(0, 1);
        dom.set_child(1, 2);
        dom.set_child(2, 3);
        dom.set_sibling(3, 4);
        dom.set_sibling(4, 5);

        // dom.set_padding(0, 1, 1, 1, 1);
        dom.set_padding(1, [2, 2, 2, 2]);
        dom.set_padding(2, [3, 3, 3, 3]);
        dom.set_padding(3, [2, 2, 2, 2]);
        dom.set_padding(4, [2, 2, 2, 2]);
        dom.set_padding(5, [2, 2, 2, 2]);        
        
        dom.set_margin(2, [1, 1, 1, 1]);        
        dom.set_margin(3, [1, 1, 1, 1]);        
        dom.set_margin(4, [1, 1, 1, 1]);        
        dom.set_margin(5, [1, 1, 1, 1]);        

        dom.node_ref_mut(2).set_direction(Direction::Vertical);
        
        dom.node_ref_mut(2).set_visibility(Visibility::Visible);

        let ascii =
        "\x03\x01 !\"#$%&'()*+,-./
0123456789:;<=>?\x00
@ABCDEFGHIJKLMNO
PQRSTUVWXYZ[\\]^
_`abcdefghijklmn
opqrstuvwxyz{|}~";
        
        let text1 = format!("\x02\x00\x04\x01\x04\x05{:05}", repeat_cnt);
        dom.set_content(3, dom::ContentContainer::Raw { raw: text1.as_str().as_ptr(), len: text1.len()} );
        
        let text1 = format!("\x01\x01\x02\x00\x04\x01{}", ascii.to_string());
        dom.set_content(4, dom::ContentContainer::Raw { raw: text1.as_str().as_ptr(), len: text1.len()} );
        dom.set_content(5, dom::ContentContainer::Raw { raw: text1.as_str().as_ptr(), len: text1.len()} );

        dom.node_ref_mut(0).set_size(Size { width: SCREEN_WIDTH, height: SCREEN_HEIGHT });
                
        dom.node_ref_mut(4).set_flex(0, 1);
        dom.node_ref_mut(5).set_flex(0, 0);
        
        dom.request_style(4).set_background(1);
        dom.request_style(4).set_color(0);
        dom.set_border(1, BORDER_RECT);
        dom.set_border(2, BORDER_RECT);
        dom.set_border(3, BORDER_RECT);        
        dom.set_border(4, BORDER_RECT);        
        dom.set_border(5, BORDER_RECT);        
        
        surface.clear(0);
        surface.update(dom);
        surface.draw_image(Identifier::Index(252), cursor, None);                
    }
}

#[tokio::main]
async fn main(){    
    Example::main().await;
}