#[cfg(test)]
extern crate std;

use crate::tests::helpers::SurfaceTest;

use crate::{Size, Area, Point};

#[test]
fn surface_draw() {
    assert_eq!(true, SurfaceTest::run(Size{width: 8, height: 8}, |surface| {
        surface.draw_rect(Area { point: Point { x: 0, y: 0 }, size: Size { width: 8, height: 8 } }, 1);
        surface.draw_pixel(Point { x: 5, y: 6 }, 1);
        surface.draw_hline(Point { x: 2, y: 2 }, 3, 1);
        surface.draw_vline(Point { x: 5, y: 2 }, 3, 1);
        surface.fill_rect(Area { point: Point { x: 2, y: 5 }, size: Size { width: 2, height: 2 } }, 1);         
    }, concat!(
        "########",
        "#      #",
        "# #### #",
        "#    # #",
        "#    # #",
        "# ##   #",
        "# ## # #",
        "########"
    )));    
}

#[test]
fn surface_line() {
    assert_eq!(true, SurfaceTest::run(Size{width: 8, height: 8}, |surface| {
        surface.draw_line(Point { x: 1, y: 1 }, Point { x: 5, y: 6 }, 1);    
    }, concat!(
         "        ",
         " #      ",
         "  #     ",
         "   #    ",
         "   #    ",
         "    #   ",
         "     #  ",
         "        "
    )));    
}