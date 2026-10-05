#[cfg(test)]
extern crate std;
use std::{string::String, vec, vec::Vec};

use crate::tests::helpers::load_xml;
use crate::{dom::DOM, media::Media, DOM_DEPTH_MAX, NODE_MAX, CONTENT_MAX, NodeIdx};

/// Runs `check` on the DOM built from `xml` by the XML loader and by the binary loader.
/// Markup built at runtime is passed as `String::leak()`, so it works with `from_xml_static` too.
fn both<F>(xml: &'static str, check: F) where F: Fn(&DOM) {
    let mut dom = DOM::new();
    load_xml(&mut dom, xml);
    check(&dom);

    let bin = Media::xml_to_bin(xml);
    let mut dom = DOM::new();
    dom.from(&bin);
    check(&dom);
}

/// Node count and deepest level (root = 0) of the tree reachable from the root.
fn walk(dom: &DOM) -> (usize, usize) {
    let (mut count, mut max_depth) = (0, 0);
    let mut stack: Vec<(NodeIdx, usize)> = vec![(0, 0)];
    while let Some((n_i, depth)) = stack.pop() {
        count += 1;
        max_depth = max_depth.max(depth);
        let node = dom.node(n_i);
        if node.next_sibling != NODE_MAX { stack.push((node.next_sibling, depth)); }
        if node.first_child != NODE_MAX { stack.push((node.first_child, depth + 1)); }
    }
    (count, max_depth)
}

fn texts(dom: &DOM) -> Vec<String> {
    (0..NODE_MAX).filter_map(|n_i| {
        let c_i = dom.node(n_i).plot.content_idx;
        if c_i == CONTENT_MAX { return None; }
        let content = &dom.contents[c_i as usize];
        Some(String::from(core::str::from_utf8(unsafe { core::slice::from_raw_parts(content.raw, content.len) }).unwrap()))
    }).collect()
}

#[test]
fn dom_too_many_nodes_drops_whole_element() {
    let mut xml = String::from("<N>");
    for _ in 1..NODE_MAX { xml.push_str("<N></N>"); }
    // Dropped: neither its attribute, its text nor its child may land on the last node that fit.
    xml.push_str("<N width=\"9\">lost<N>deeper</N></N>");
    xml.push_str("</N>");
    both(xml.leak(), |dom| {
        assert_eq!(walk(dom).0, NODE_MAX as usize);
        assert_ne!(dom.node(NODE_MAX - 1).plot.size.width, 9);
        assert!(texts(dom).is_empty(), "{:?}", texts(dom));
    });
}

#[test]
fn dom_too_deep_drops_subtree_and_resumes() {
    let mut xml = String::from("<N>");
    for _ in 1..DOM_DEPTH_MAX + 2 { xml.push_str("<N>"); }
    xml.push_str("deep");
    for _ in 1..DOM_DEPTH_MAX + 2 { xml.push_str("</N>"); }
    // Parsing resumes normally once the dropped subtree is closed.
    xml.push_str("<N>after</N></N>");
    both(xml.leak(), |dom| {
        assert_eq!(walk(dom), (DOM_DEPTH_MAX + 1, DOM_DEPTH_MAX - 1));
        assert_eq!(texts(dom), vec![String::from("after")]);
    });
}

#[test]
fn dom_extra_close_tags_no_underflow() {
    both("<N><N>a</N></N></N></N>", |dom| {
        assert_eq!(walk(dom).0, 2);
    });
}
