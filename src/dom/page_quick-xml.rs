use log;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use crate::*;
use crate::dom::*;
use crate::node::*;
use crate::font::*;

pub fn from_xml (dom: &mut DOM, xml: &str) {
    let mut depth = 0;
    let mut p_stack = [NODE_MAX as u8; DOM_DEPTH_MAX];
    let mut n_i = 0;

    dom.reset();
    dom.nodes[0].size.width = SCREEN_WIDTH;
    dom.nodes[0].size.height = SCREEN_HEIGHT;

    log::debug!("parsing xml");

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    reader.config_mut().allow_unmatched_ends = true;

    loop {                
        let position = reader.buffer_position();
        let event = reader.read_event();        
        if event.is_err() {
            log::error!("xml error at {}", reader.error_position());
            break;
        }
        match event.unwrap() {
            Event::Start(start) => {
                let mut split = str::from_utf8(&start).unwrap_or("").split(" ");                                

                if depth > 0 {
                    n_i += 1;                    
                    let c_i = dom.nodes[p_stack[depth - 1] as usize].first_child;
                    if c_i == NODE_MAX {
                        dom.set_child(p_stack[depth - 1], n_i);
                    }else{
                        let mut s_i = c_i; 
                        loop {
                            let ns_i = dom.nodes[s_i as usize].next_sibling;
                            if ns_i == NODE_MAX {
                                dom.nodes[s_i as usize].next_sibling = n_i;
                                break;
                            }else{
                                s_i = ns_i;
                            }
                        }
                    }
                }                  
                
                let name = split.next().unwrap_or("");
                log::debug!("{position:4}| ({depth}:{n_i}) <{name}>");              

                let dir_p = if depth > 0 { dom.node_ref(p_stack[depth - 1]).direction() } else { Direction::Horizontal };                
                loop {
                    let opt = split.next();
                    if opt.is_some() {
                       let mut attr = opt.unwrap().split("=");
                       let key = attr.next().unwrap_or("");
                       let mut value = if opt.unwrap().len() > key.len() + 1 { &opt.unwrap()[key.len() + 1 ..] } else { "" };
                       if value.len() >= 2 && value.starts_with("\"") && value.ends_with("\"") {
                        value = &value[1 .. value.len() - 1];
                       }
                       log::debug!("{position:4}| ({depth}:{n_i})  {key} = \"{value}\"");
                       match key {                        
                        "vertical"=>{
                            dom.node_ref_mut(n_i).set_direction(Direction::Vertical);
                        }
                        "width"=>{
                            let res = value.parse::<SizeW>();
                            if res.is_ok() {
                                dom.node_ref_mut(n_i).set_width(res.unwrap());
                            }else{
                                log::error!("illegal width \"{value}\"");
                            }
                        }
                        "height"=>{
                            let res = value.parse::<SizeH>();
                            if res.is_ok() {
                                dom.node_ref_mut(n_i).set_height(res.unwrap());
                            }else{
                                log::error!("illegal height \"{value}\"");
                            }
                        }
                        "align"=>{
                            if depth > 0 {                                
                                match value {
                                    "start"   =>{}
                                    "center"  =>{ dom.node_ref_mut(n_i).set_alignment(Alignment::Center); }
                                    "end"     =>{ dom.node_ref_mut(n_i).set_alignment(Alignment::End); }
                                    "stretch" =>{ dom.node_ref_mut(n_i).set_alignment(Alignment::Stretch); }                                        
                                    _=>{
                                        log::error!("illegal align \"{value}\"")
                                    }
                                }
                            }
                        }                        
                        "visible"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );
                            dom.node_ref_mut(n_i).set_visibility(if value != 0 { Visibility::Visible } else { Visibility::Hidden });                            
                        }
                        "margin"=>{
                            let values: Offset = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );
                            if values > 0 {
                                dom.set_margin(n_i, values, values, values, values);
                            }
                        }
                        "padding"=>{                            
                            let values: Offset = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );                            
                            if values > 0 {
                                dom.set_padding(n_i, values, values, values, values);
                            }
                        }                        
                        "background"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );                            
                            dom.request_style(n_i).set_background(value);
                        }
                        "color"=>{
                            let value: u8 = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 1; } );                                                        
                            dom.request_style(n_i).set_color(value);
                        }
                        "border"=>{                                                                                    
                            dom.request_style(n_i).set_border(BORDER_RECT);                          
                        }
                        "font"=>{                                          
                            let value: FontIdx = value.parse().unwrap_or_else(|e| { log::error!("{}", e); return 0; } );                                                  
                            dom.request_style(n_i).set_font(value);                          
                        }
                        _=>{}
                    }
                       
                    } else {
                        break;                    
                    }
                }

                p_stack[depth] = n_i;
                depth += 1;
            },            
            Event::End(end) => {
                let name = str::from_utf8(&end).unwrap().split(" ").next().unwrap_or("");
                depth -= 1;
                log::debug!("{position:4}| ({depth}:{n_i}) </{name}>");
            },
            Event::Text(text) => {
                let raw = text.as_ptr();
                let str = str::from_utf8(&text).unwrap_or_else(|e| { log::error!("{}", e); return "" });
                if dom.node(n_i).content_idx == CONTENT_MAX && dom.node(n_i).first_child == NODE_MAX {
                    dom.set_content(n_i, Content::Raw{ raw: raw, len : str.len() });
                    log::debug!("{position:4}| ({depth}:{n_i})  Content: \"{str}\"");
                }
                
            },            
            Event::Eof => break,            
            _ => {}
        }
    }    
}