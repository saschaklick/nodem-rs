#[cfg(test)]
extern crate std;
use std::{string::String, vec, vec::Vec};

use crate::{control::{Control, ControlMode, IControl, LoaderRet}, surface::Surface};
use crate::tests::helpers::PKG_SYS;

fn send(control: &mut Control, surface: &mut Surface, buf: &[u8]) -> String {
    let mut listeners: [Option<&mut dyn IControl>; 4] = [None, None, None, None];
    let mut res = String::new();
    control.process(buf, surface, &mut listeners, &mut res);
    res
}

fn header(len: usize) -> Vec<u8> {
    let mut buf = b"pkg\n".to_vec();
    buf.extend_from_slice(&(len as u32).to_le_bytes());
    buf
}

#[test]
fn control_split_header_progress_no_panic() {
    let mut fb = vec![0u8; 8 * 8 / 8];
    let mut surface = Surface::new(fb.as_mut_slice(), 8, 8);
    let mut control = Control::default();
    send(&mut control, &mut surface, b"pkg\n");
    // 1024 bytes: the first header byte is 0x00, so the partial size is 0.
    send(&mut control, &mut surface, &[0x00]);
    assert!(control.is_loader_busy());
    assert_eq!(control.get_loader_progress(255), None);
    send(&mut control, &mut surface, &[0x04, 0x00, 0x00]);
    assert_eq!(control.get_loader_progress(255), Some(0));
}

#[test]
fn control_empty_package_rejected_rest_is_commands() {
    let mut fb = vec![0u8; 8 * 8 / 8];
    let mut surface = Surface::new(fb.as_mut_slice(), 8, 8);
    let mut control = Control::default();
    let mut buf = header(0);
    buf.extend_from_slice(b"info\n");
    let res = send(&mut control, &mut surface, &buf);
    assert!(res.starts_with("pkg3\r\n"), "{res}");
    assert!(res.contains(env!("CARGO_PKG_NAME")), "{res}");
    assert!(matches!(control.mode, ControlMode::LineMode));
    assert!(matches!(control.get_loader_error(), LoaderRet::NotEnoughSpace));
    assert_eq!(control.get_loader_progress(255), None);
}

#[test]
fn control_package_then_command_in_one_buffer() {
    let mut fb = vec![0u8; 8 * 8 / 8];
    let mut surface = Surface::new(fb.as_mut_slice(), 8, 8);
    let mut control = Control::default();
    let mut buf = header(PKG_SYS.len());
    buf.extend_from_slice(PKG_SYS);
    buf.extend_from_slice(b"info\n");
    let res = send(&mut control, &mut surface, &buf);
    assert!(res.contains("pkg0\r\n"), "{res}");
    assert!(res.contains(env!("CARGO_PKG_NAME")), "{res}");
    assert!(!control.is_loader_busy());
}

#[test]
fn control_new_pkg_clears_previous_error() {
    let mut fb = vec![0u8; 8 * 8 / 8];
    let mut surface = Surface::new(fb.as_mut_slice(), 8, 8);
    let mut control = Control::default();

    // Rejected (empty) and failed (corrupt) transfers leave their error behind: the loader stays "busy".
    let res = send(&mut control, &mut surface, &header(0));
    assert!(res.contains("pkg3\r\n"), "{res}");
    assert!(control.is_loader_busy());
    let mut buf = header(16);
    buf.extend_from_slice(&[0u8; 16]);
    let res = send(&mut control, &mut surface, &buf);
    assert!(res.contains("pkg4\r\n"), "{res}");
    assert!(control.is_loader_busy());

    // A new transfer starts clean.
    send(&mut control, &mut surface, b"pkg\n");
    assert!(matches!(control.get_loader_error(), LoaderRet::Ok));
    assert!(!control.is_loader_busy());
    let mut buf = (PKG_SYS.len() as u32).to_le_bytes().to_vec();
    buf.extend_from_slice(PKG_SYS);
    let res = send(&mut control, &mut surface, &buf);
    assert!(res.contains("pkg0\r\n"), "{res}");
    assert!(!control.is_loader_busy());
}
