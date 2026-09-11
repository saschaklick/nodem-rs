#[cfg(test)]
extern crate std;

use crate::tests::helpers::DOMTest;

use crate::{Size};

#[test]
fn dom_content() {
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N>Ai1.</N>");     
    }, concat!(
        " #  # ##        ",
        "# #    #        ",
        "### #  #        ",
        "# # # ### #     ",
        "                ",
        "                ",
        "                ",        
        "                ",        
        "                ",        
        "                "
    )));   

    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N>{mono}Ai1.</N>");     
    }, concat!(
        " #   #  ##      ",
        "# #      #      ",
        "###  #   #      ",
        "# #  #  ###  #  ",
        "                ",
        "                ",
        "                ",        
        "                ",        
        "                ",        
        "                "
    )));   
}

#[test]
fn dom_border() {
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N border=solid padding=\"3 4\"></N>");     
    }, concat!(
        "############    ",
        "#          #    ",
        "#          #    ",
        "#          #    ",
        "#          #    ",
        "#          #    ",
        "#          #    ",                        
        "#          #    ",
        "#          #    ",
        "############    "
    )));        
}

#[test]
fn dom_margin() {
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N border=solid padding=2 margin=\"2 1\"></N>");     
    }, concat!(
        "                ",        
        "                ",        
        " ########       ",
        " #      #       ",
        " #      #       ",
        " #      #       ",
        " #      #       ",
        " #      #       ",
        " #      #       ",        
        " ########       "
    )));    
}

#[test]
fn dom_flex() {
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N width=flex border=solid padding=2></N>");     
    }, concat!(        
        "################",
        "#              #",
        "#              #",
        "#              #",
        "#              #",
        "#              #",
        "#              #",        
        "################",        
        "                ",
        "                "
    )));        
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N height=flex border=solid padding=2></N>");     
    }, concat!(        
        "########        ",
        "#      #        ",
        "#      #        ",
        "#      #        ",
        "#      #        ",
        "#      #        ",
        "#      #        ",        
        "#      #        ",        
        "#      #        ",
        "########        "
    )));        
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 10}, |dom| {
        dom.from_xml("<N width=flex height=flex border=solid padding=2></N>");     
    }, concat!(        
        "################",
        "#              #",
        "#              #",
        "#              #",
        "#              #",
        "#              #",
        "#              #",        
        "#              #",        
        "#              #",
        "################"
    )));      
}

#[test]
fn dom_align() {
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 16}, |dom| {
        dom.from_xml("
            <N width=flex height=flex>
                <N align=start border=solid></N>
                <N align=center border=solid></N>
                <N align=end border=solid></N>
            </N>"
        );     
    }, concat!(        
        "####            ",
        "#  #            ",
        "#  #            ",
        "####            ",
        "                ",
        "                ",
        "    ####        ",
        "    #  #        ",
        "    #  #        ",        
        "    ####        ",        
        "                ",
        "                ",
        "        ####    ",
        "        #  #    ",
        "        #  #    ",
        "        ####    ",        
    )));       
    
    assert_eq!(true, DOMTest::run(Size {width: 16, height: 16}, |dom| {
        dom.from_xml("
            <N width=flex height=flex vertical>
                <N align=start border=solid></N>
                <N align=center border=solid></N>
                <N align=end border=solid></N>
            /N>"
        );     
    }, concat!(        
        "####            ",
        "#  #            ",
        "#  #            ",
        "####            ",
        "      ####      ",
        "      #  #      ",
        "      #  #      ",
        "      ####      ",
        "            ####",        
        "            #  #",        
        "            #  #",
        "            ####",
        "                ",
        "                ",
        "                ",
        "                ",        
    )));  
}

#[test]
fn dom_usecases() {
    assert_eq!(true, DOMTest::run(Size {width: 40, height: 24}, |dom| {
        dom.from_xml("
            <N width=flex height=flex>
                <N width=flex height=flex background=1>
                    <N width=flex background=0 padding=1 align=start>S</N>
                    <N width=flex background=0 padding=1 align=center>C</N>
                    <N width=flex background=0 padding=1 align=end>E</N>
                    <N width=flex background=0 padding=1 align=stretch>T</N>
                </N>
                <N width=flex height=flex background=0 vertical>
                    <N height=flex background=1 color=0 padding=1 align=start>S</N>
                    <N height=flex background=1 color=0 padding=1 align=center>C</N>
                    <N height=flex background=1 color=0 padding=1 align=end>E</N>
                    <N height=flex background=1 color=0 padding=1 align=stretch>T</N>
                </N>
            </N>"
        );     
    }, concat!(        
            "     ##########     #####               ",
            " ### ##########     #   #               ",
            " ##  ##########     #  ##               ",
            "   # ##########     ### #               ",
            " ### ##########     #   #               ",
            "     ##########     #####               ",
            "###############            #####        ",
            "###############            ##  #        ",                                         
            "###############            # ###        ",                                         
            "#####     #####            # ###        ",                                         
            "#####  ## ##### ###        ##  #        ",                                         
            "##### #   #####  #         #####        ",                                         
            "##### #   #####  #                 #####",                                         
            "#####  ## #####  #                 #   #",                                         
            "#####     #####                    #  ##",                                         
            "###############                    # ###",                                         
            "###############                    #   #",                                         
            "###############                    #####",                                         
            "##########          ####################",                                         
            "########## ###      ########   #########",                                         
            "########## ##       ######### ##########",                                         
            "########## #        ######### ##########",                                         
            "########## ###      ######### ##########",                                         
            "##########          ####################",
    )));  
    assert_eq!(true, DOMTest::run(Size {width: 48, height: 32}, |dom| {
        dom.from_xml("
            <N width=flex height=flex>
                <N width=flex height=flex background=1>
                    <N width=flex background=0 padding=1 align=start>S</N>
                    <N width=flex background=0 padding=1 align=center>C</N>
                    <N width=flex background=0 padding=1 align=end>E</N>
                    <N width=flex background=0 padding=1 align=stretch>T</N>
                </N>
                <N width=flex height=flex background=0 vertical>
                    <N height=flex background=1 color=0 padding=1 align=start>S</N>
                    <N height=flex background=1 color=0 padding=1 align=center>C</N>
                    <N height=flex background=1 color=0 padding=1 align=end>E</N>
                    <N height=flex background=1 color=0 padding=1 align=stretch>T</N>
                </N>
            </N>"
        );     
    }, concat!(        
        "      ############      #####                   ",
        " ###  ############      #####                   ",
        " ##   ############      #   #                   ",
        "   #  ############      #  ##                   ",
        " ###  ############      ### #                   ",
        "      ############      #   #                   ",
        "##################      #####                   ",
        "##################      #####                   ",
        "##################               #####          ",
        "##################               #####          ",
        "##################               ##  #          ",
        "##################               # ###          ",
        "##################               # ###          ",
        "######      ######               ##  #          ",
        "######  ##  ###### ###           #####          ",
        "###### #    ######  #            #####          ",
        "###### #    ######  #                      #####",
        "######  ##  ######  #                      #####",
        "######      ######                         #   #",
        "##################                         #  ##",
        "##################                         # ###",                                                 
        "##################                         #   #",                                                 
        "##################                         #####",                                                 
        "##################                         #####",                                                 
        "##################      ########################",                                                 
        "##################      ########################",                                                 
        "############            ##########   ###########",                                                 
        "############ ###        ########### ############",                                                 
        "############ ##         ########### ############",                                                 
        "############ #          ########### ############",                                                 
        "############ ###        ########################",                                                 
        "############            ########################"
    )));        
}