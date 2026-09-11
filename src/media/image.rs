use crate::media::*;

pub use u8 as ImageIdx;
pub use u8 as ImageW;
pub use u8 as ImageH;
pub use u8 as ImageFormat;

#[repr(u8)]
pub enum ImageType {
    Dummy    = 0x00,
    Indexed1 = 0x01,
    // Indexed2 = 0x02,
    // Indexed4 = 0x03,
    // Indexed8 = 0x04,
    // EOL
}

pub const IMAGE_HEADER_SIZE: usize = 3;

static DUMMY_IMAGE: [u8;8] = [ ImageType::Indexed1 as u8, 0x05, 0x05, Encoding::Raw as u8, 0b11111110, 0b01101011, 0b00111111, 0b00000000 ];

pub struct Image<'a> {        
    pub source: u8,
    pub id: &'a str,
    pub data: &'a [u8],
    pub mask: Option<&'a [u8]>
}
impl Default for Image<'_> {
    fn default() -> Self { Image { source: 255, id: "", data: &DUMMY_IMAGE, mask : None } }
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub transparent_color: Color,
    pub use_mask: bool,
    pub inverted: bool    
}
impl Default for Settings {
    fn default() -> Self { Settings { transparent_color: 0, use_mask: true, inverted: false } }
}

impl Media {
    pub fn get_image(&self, ident: Identifier) -> Image <'_> {            
        let mut lib_idx = self.images.len() as u8;
        for lib in self.images.iter().rev() {
            lib_idx -= 1;
            if lib.data == ptr::null() {                
                continue;
            }            
            let data = unsafe { slice::from_raw_parts(lib.data.as_ref().unwrap(), lib.len) };                        
            let mut reader = Stream::new(&data); 
            loop {                
                if reader.available() >= 2 {                                    
                    let idx = reader.read_u8();
                    let id_len = reader.read_u8() as usize;
                    if reader.available() >= id_len {
                        let id = str::from_utf8(&data[reader.get_pos() .. reader.get_pos() + id_len]).unwrap_or("");
                        reader.set_pos(reader.get_pos() + id_len);
                        if reader.ok == false || idx == 255 {                                        
                            break;
                        }                
                        if reader.available() >= IMAGELIB_HEADER_SIZE - 1 {             
                            let image_length= reader.read_u16() as usize;
                            let mask_length = reader.read_u16() as usize;                                
                            if reader.available() > image_length + mask_length {                                                                               
                                if match ident {                                        
                                    Identifier::Index(index) => index == idx,
                                    Identifier::Name(name) => !name.is_empty() && name == id,
                                    Identifier::Both(index, name) => index == idx || (!name.is_empty() && name == id)
                                } {
                                    let pos = reader.get_pos();
                                    return if image_length == 0 { Image::default() } else {
                                        Image {
                                            source: lib_idx,                                                                        
                                            id: id,
                                            data: &data[pos .. pos + image_length],                                    
                                            mask: if mask_length == 0 { None } else { Some(&data[pos + image_length .. pos + image_length + mask_length]) }
                                        }
                                    }
                                }                            
                            }
                            reader.set_pos(reader.get_pos() + image_length + mask_length);                
                        }else{
                            break;
                        }                    
                    }
                }else{
                    break;
                }
            }            
        }

        return Image::default();
    }
}