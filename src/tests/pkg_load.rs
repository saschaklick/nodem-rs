//! `Media::load_pkg` on complete vs. truncated packages. The writer (media/pkg.rs) ends every package
//! with a single LIBMAGIC_EOL byte; the reader must accept that as the end instead of reporting
//! UnexpectedEnd, while a package that stops without its EOL must still be reported.

extern crate std;
use std::vec::Vec;
use crate::media::*;
use crate::tests::helpers::PKG_SYS;

/// "PKG0" + u32 length + u16 CRC (seed 0x1002, sum of bytes from offset 12), then `body`.
fn package(body: &[u8]) -> Vec<u8> {
    let mut pkg = b"PKG0".to_vec();
    let len = (10 + body.len()) as u32;
    pkg.extend_from_slice(&len.to_ne_bytes());
    pkg.extend_from_slice(&[0, 0]);
    pkg.extend_from_slice(body);
    let mut crc: u16 = 0x1002;
    for b in &pkg[12..] {
        crc = crc.wrapping_add(*b as u16);
    }
    pkg[8..10].copy_from_slice(&crc.to_ne_bytes());
    pkg
}

fn load(pkg: &[u8]) -> u8 {
    let mut media = Media::default();
    media.load_pkg(pkg.as_ptr(), pkg.len(), 2) as u8
}

#[test]
fn complete_package_loads_ok() {
    // One 1-byte image library, then the EOL byte the writer always appends.
    let pkg = package(&[LIBMAGIC_IMAGE, 1, 0, 0xff, LIBMAGIC_EOL]);
    assert_eq!(load(&pkg), Ret::Ok as u8);
}

#[test]
fn package_without_eol_is_unexpected_end() {
    let pkg = package(&[LIBMAGIC_IMAGE, 1, 0, 0xff]);
    assert_eq!(load(&pkg), Ret::UnexpectedEnd as u8);
}

#[test]
fn system_package_loads_ok() {
    // The resident system package every device carries was built by the same writer.
    assert_eq!(load(PKG_SYS), Ret::Ok as u8);
}

#[cfg(feature = "inspect")]
#[test]
fn inspect_lists_font_glyph_ranges() {
    let mut media = Media::default();
    media.load_pkg(PKG_SYS.as_ptr(), PKG_SYS.len(), 0);
    let mut csv = std::string::String::new();
    media.inspect(255, &mut csv).unwrap();
    let mut lines = csv.lines().skip_while(|l| !l.starts_with("font_idx,"));
    assert_eq!(lines.next(), Some("font_idx,id,source,height,base,mono,glyphs,ranges"));
    let row = lines.next().unwrap();
    let cols: Vec<&str> = row.splitn(8, ',').collect();
    assert_eq!(cols.len(), 8, "{row}");
    // `glyphs` is the sum of the listed ranges.
    let ranges = cols[7].trim_matches('"');
    let sum: u32 = ranges.split(' ').map(|r| {
        let (a, b) = r.split_once('-').unwrap();
        b.parse::<u32>().unwrap() - a.parse::<u32>().unwrap() + 1
    }).sum();
    assert!(sum > 0);
    assert_eq!(cols[6].parse::<u32>().unwrap(), sum, "{row}");
}
