#[cfg(test)]
extern crate std;
use std::{string::String, vec};
use crate::{surface::Surface, dom::DOM};
use crate::tests::helpers::{PKG_SYS, load_xml};

#[test]
fn dom_real_menu_render() {
    // A realistic menu board (~20 nodes, 19 texts with umlauts and ß) fits the default limits and renders.
    let vals = [
        "Tageskarte Mittwoch, 16.09.2026","K\u{fc}rbiscremesuppe","mit ger\u{f6}steten Kernen und Ingwer","6,50 EUR",
        "Rinderroulade","mit Rotkohl und Kartoffelkl\u{f6}\u{df}en","16,90 EUR","Schnitzel Wiener Art",
        "mit Pommes frites und Salatbeilage","14,90 EUR","Gebratenes Zanderfilet","auf Blattspinat mit Petersilienkartoffeln",
        "18,50 EUR","Gem\u{fc}securry","vegan, mit Basmatireis und Koriander","12,50 EUR","Apfelstrudel","mit Vanillesauce","5,90 EUR"];
    let mut markup = String::from("<body vertical>");
    for v in vals { markup.push_str("<div>"); markup.push_str(v); markup.push_str("</div>"); }
    markup.push_str("</body>");

    let (w, h) = (250usize, 122usize);
    let mut buf = vec![0u8; (w * h + 7) / 8];
    let mut s = Surface::new(buf.as_mut_slice(), w as u16, h as u16);
    s.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
    let mut dom = DOM::new();
    load_xml(&mut dom, markup.leak());
    assert_eq!(dom.content_idx as usize, vals.len());
    s.update(&mut dom);
}
