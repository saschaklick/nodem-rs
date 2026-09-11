#[derive(Copy, Clone)]
pub struct Id {
    pub raw: *const u8,
    pub len: usize
}

impl Default for Id {
    fn default() -> Self { 
        Id {
            raw: core::ptr::null(),
            len: 0
        }
    }
}

impl Id {
}
