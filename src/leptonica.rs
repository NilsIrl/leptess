use super::capi;

pub struct Pix {
    pub raw: *mut capi::Pix,
}

pub enum FileFormat {
    Unknown,
    Bmp,
    JfifJpeg,
    Png,
    Tiff,
    TiffPackbits,
    TiffRle,
    TiffG3,
    TiffG4,
    TiffLzw,
    TiffZip,
    Pnm,
    Ps,
    Gif,
    Jp2,
    Webp,
    Lpdf,
    TiffJpeg,
    Default,
    Spix,
}

impl FileFormat {
    // https://github.com/DanBloomberg/leptonica/blob/95405007f7ebf7df69f13475b3259179cdc4ec12/src/imageio.h#L91
    fn to_int(&self) -> i32 {
        match self {
            FileFormat::Unknown => 0,
            FileFormat::Bmp => 1,
            FileFormat::JfifJpeg => 2,
            FileFormat::Png => 3,
            FileFormat::Tiff => 4,
            FileFormat::TiffPackbits => 5,
            FileFormat::TiffRle => 6,
            FileFormat::TiffG3 => 7,
            FileFormat::TiffG4 => 8,
            FileFormat::TiffLzw => 9,
            FileFormat::TiffZip => 10,
            FileFormat::Pnm => 11,
            FileFormat::Ps => 12,
            FileFormat::Gif => 13,
            FileFormat::Jp2 => 14,
            FileFormat::Webp => 15,
            FileFormat::Lpdf => 16,
            FileFormat::TiffJpeg => 17,
            FileFormat::Default => 18,
            FileFormat::Spix => 19,
        }
    }
}

impl Pix {
    // TODO: read from std::fs::File
    pub fn from_path(path: &std::path::Path) -> Pix {
        Pix {
            raw: unsafe {
                let pix = capi::pixRead(
                    std::ffi::CString::new(path.to_str().unwrap())
                        .unwrap()
                        .as_ptr(),
                );
                if pix.is_null() {
                    panic!("Invalid file");
                }
                pix
            },
        }
    }

    pub fn clip(&self, rectangle: &Box) -> Self {
        Pix {
            raw: unsafe {
                let pix = capi::pixClipRectangle(self.raw, rectangle.raw, std::ptr::null_mut());
                if pix.is_null() {
                    panic!("pixClipRectangle returned NULL");
                }
                pix
            },
        }
    }

    // TODO: what to ask for, a &str, path or file?
    pub fn write(&self, path: &std::path::Path, format: FileFormat) -> Result<(), ()> {
        if unsafe {
            // https://github.com/DanBloomberg/leptonica/blob/95405007f7ebf7df69f13475b3259179cdc4ec12/src/writefile.c#L341
            capi::pixWrite(
                std::ffi::CString::new(path.to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
                self.raw,
                format.to_int(),
            ) == 0
        } {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Drop for Pix {
    fn drop(&mut self) {
        unsafe {
            capi::pixDestroy(&mut self.raw);
        }
    }
}

pub struct Box {
    pub raw: *mut capi::Box,
}

impl Drop for Box {
    fn drop(&mut self) {
        unsafe {
            capi::boxDestroy(&mut self.raw);
        }
    }
}

impl Box {
    /// The x position the box
    pub fn x(&self) -> i32 {
        unsafe { (*self.raw).x }
    }
    /// The y position of the box
    pub fn y(&self) -> i32 {
        unsafe { (*self.raw).y }
    }
    /// The width of the box
    pub fn w(&self) -> i32 {
        unsafe { (*self.raw).w }
    }
    /// The height of the box
    pub fn h(&self) -> i32 {
        unsafe { (*self.raw).h }
    }
}

pub struct Boxes {
    pub raw: *mut capi::Boxa,
}

impl Drop for Boxes {
    fn drop(&mut self) {
        unsafe {
            capi::boxaDestroy(&mut self.raw);
        }
    }
}

impl Boxes {
    // https://github.com/rust-lang/rfcs/issues/1791
    pub fn len(&self) -> usize {
        unsafe { (*self.raw).n as usize }
    }

    pub fn get(&self, index: usize) -> Box {
        unsafe {
            let b = capi::boxaGetBox(self.raw, index as i32, capi::L_CLONE as i32);
            if b.is_null() {
                panic!("Found null box");
            }
            Box { raw: b }
        }
    }
}

impl IntoIterator for Boxes {
    type Item = Box;
    type IntoIter = BoxesIterator;

    fn into_iter(self) -> Self::IntoIter {
        let count = self.len();
        BoxesIterator {
            boxa: self,
            index: 0,
            count: count,
        }
    }
}

// TODO: tesseract offers a direct iterator
pub struct BoxesIterator {
    boxa: Boxes,
    index: usize,
    count: usize,
}

impl Iterator for BoxesIterator {
    type Item = Box;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let re = self.boxa.get(self.index);
        self.index += 1;

        Some(re)
    }
}

impl<'a> IntoIterator for &'a Boxes {
    type Item = Box;
    type IntoIter = BoxaRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let count = self.len();
        BoxaRefIterator {
            boxa: self,
            index: 0,
            count: count,
        }
    }
}

pub struct BoxaRefIterator<'a> {
    boxa: &'a Boxes,
    index: usize,
    count: usize,
}

impl<'a> Iterator for BoxaRefIterator<'a> {
    type Item = Box;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let re = self.boxa.get(self.index);
        self.index += 1;

        Some(re)
    }
}
