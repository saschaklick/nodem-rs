use crate::{control::{ Control, IControl, decode_values }, dom::{ DOM, Ret } , media::Media, node::{ HEIGHT_CONTENT, WIDTH_CONTENT }, * };
#[cfg(feature = "alloc")]
use crate::dom::ContentContainer;

impl IControl for DOM {
    fn process_line(&mut self, line: &str, media: &Media, res: &mut dyn core::fmt::Write) -> (bool, core::fmt::Result) {                
        let prefix = "$";
        if line.starts_with(prefix) {            
            let dom = self;            
            let ret = match line[1..].as_ref() {
                "clear" => { dom.clear(); Ret::Ok },
                "xml" => { dom.to_xml(res, Some(" "), false).expect(""); Ret::Ok }
                "inspect" => { dom.inspect(res).expect(""); Ret::Ok }
                _ => {                    
                    let split_p = line.find("=").unwrap_or(line.len());
                    let mut split = line[prefix.len()..split_p].split(".");
                    let node = split.next();
                    let prop = split.next();
                    let value = if split_p >= line.len() { " " } else { line[split_p + 1..].trim() };
                    if node.is_some() && prop.is_some() {
                        let node_str = node.unwrap();
                        let node_i = node_str.parse::<NodeIdx>().unwrap_or(NODE_MAX);
                        let values = decode_values::<Offset>(value);
                        if node_i < NODE_MAX {  
                            let node = &mut dom.nodes[node_i as usize];
                            let mut ret = Ret::Ok;
                            match prop.unwrap_or("").trim() {                                                                                                                    
                                "child"      => { let _ = value.parse::<NodeIdx>().map(|value|{ dom.set_child(node_i as NodeIdx, value); }); }
                                "sibling"    => { let _ = value.parse::<NodeIdx>().map(|value|{ dom.set_sibling(node_i as NodeIdx, value); }); }
                                "remove"     => { dom.remove(node_i); }
                                #[cfg(feature = "alloc")]
                                "content"    => { dom.set_content(node_i, ContentContainer::Str(value)); }
                                #[cfg(not(feature = "alloc"))]
                                "content"    => { ret = Ret::NoAllocFeature; }
                                "width"      => { if value == "content" { node.set_width(WIDTH_CONTENT); } else { let _ = value.parse::<SizeW>().map(|value|{ node.set_width(value);  }); } }
                                "height"     => { if value == "content" { node.set_height(HEIGHT_CONTENT); } else { let _ = value.parse::<SizeH>().map(|value|{ node.set_height(value); }); } }                                    
                                "padding"    => { dom.set_padding(node_i, values); }
                                "margin"     => { dom.set_margin(node_i, values); }                            
                                "direction"  => { node.set_direction( if values[0] == 0 { node::Direction::Horizontal } else { node::Direction::Vertical }); }
                                "align"      => { node.set_alignment(match values[0] {
                                    1 => node::Alignment::Center,
                                    2 => node::Alignment::End,
                                    3 => node::Alignment::Stretch,
                                    _ => node::Alignment::Start
                                }); }
                                "visible"    => { node.set_visibility( if values[0] == 0 { node::Visibility::Hidden } else { node::Visibility::Visible }); }
                                "font"       => { let _ = value.parse::<FontIdx>().map(|value|{ dom.request_style(node_i).set_font(value); }); }
                                "color"      => { let _ = value.parse::<Color>().map(|value|{ dom.request_style(node_i).set_color(value); }); }
                                "background" => { let _ = value.parse::<Color>().map(|value|{ dom.request_style(node_i).set_background(value); }); }
                                "border"     => {                                
                                    if value.eq_ignore_ascii_case("solid") {
                                        dom.request_style(node_i).set_border(BORDER_RECT);
                                    }else{
                                        let _ = value.parse::<BorderIdx>().map(|value|{ dom.request_style(node_i).set_border(value); });
                                    }                                        
                                }
                                _ => { ret = Ret::InvalidProperty; }
                            }
                            ret                                                
                        } else { Ret::InvalidArgument }               
                    } else { Ret::InvalidFormat }                                                               
                }
            };
            Control::send_result(prefix, ret as u8, res)            
        }else{                                
            let split = line.split_once('=').unwrap_or(("", ""));            
            let ret = match split.0.trim() {                
                "pkg" => { self.clear(); return (false, Ok(())); }
                #[cfg(all(feature = "dom", feature = "xml", feature = "alloc"))]
                "xml" => {
                    self.from_xml(split.1.trim())
                }, 
                #[cfg(not(feature = "alloc"))]
                "xml" => {
                    Ret::NoAllocFeature
                }
                #[cfg(feature = "dom")]
                "bin" => {
                    self.from(split.1.trim().as_bytes())                    
                },     
                #[cfg(feature = "dom")]
                "page" => match split.1.trim().parse::<u8>() {
                    Ok(value) => {
                        let page = media.get_page(value);
                        self.from(page.content);
                        match page.source {
                            255 => Ret::InvalidSource,
                            _ => Ret::Ok
                        }
                    },
                    Err(_) => Ret::InvalidArgument                              
                }                
                _ => { return (false, Ok(())); }
            };
            Control::send_result("", ret as u8, res)            
        }        
    }
}