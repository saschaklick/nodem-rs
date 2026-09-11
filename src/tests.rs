mod helpers;
mod surface;
mod dom;

// #[test]
// fn dom_basic() {    
//     let mut generated: [u8; 8] = [0u8;8];
//     let mut surface = Surface::new(&mut generated, 8,8);
//     surface.media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
//     let mut dom = DOM::new();
//     dom.from_xml("<N vertical><N >@</></N>");    
//     surface.update(&dom);

//     let expected = str_to_bitmap_8(concat!(
//         "###     ",
//         "###     ",
//         "###     ",
//         "###     ",
//         "        ",
//         "        ",
//         "        ",
//         "        "
//     ));

//     assert_eq!(true, compare_8x8(generated, expected));    
// }

