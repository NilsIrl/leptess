pub struct Pix {
    pub raw: *mut leptonica_sys::Pix,
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
    // TODO: solved by https://github.com/ccouzens/leptonica-sys/pull/2
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
    pub fn from_path(path: &std::path::Path) -> Result<Pix, ()> {
        let pix = unsafe {
            leptonica_sys::pixRead(
                std::ffi::CString::new(path.to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
            )
        };
        if pix.is_null() {
            Err(())
        } else {
            Ok(Pix { raw: pix })
        }
    }

    pub fn clip(&self, rectangle: &Box) -> Self {
        Pix {
            raw: unsafe {
                let pix =
                    leptonica_sys::pixClipRectangle(self.raw, rectangle.raw, std::ptr::null_mut());
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
            leptonica_sys::pixWrite(
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

    pub fn w(&self) -> u32 {
        unsafe { (*self.raw).w }
    }

    pub fn h(&self) -> u32 {
        unsafe { (*self.raw).h }
    }
}

impl Drop for Pix {
    fn drop(&mut self) {
        unsafe {
            leptonica_sys::pixDestroy(&mut self.raw);
        }
    }
}

pub struct Box {
    pub raw: *mut leptonica_sys::Box,
}

impl Drop for Box {
    fn drop(&mut self) {
        unsafe {
            leptonica_sys::boxDestroy(&mut self.raw);
        }
    }
}

impl Box {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Box {
        assert!(w > 1 || h > 1);
        // https://tpgit.github.io/Leptonica/boxbasic_8c.html#ad846c5f00e3aaed3dd4329347acac89d
        // Virutally impossible to get null pointer
        Box {
            raw: unsafe { leptonica_sys::boxCreate(x, y, w, h) },
        }
    }

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
    pub raw: *mut leptonica_sys::Boxa,
}

impl Drop for Boxes {
    fn drop(&mut self) {
        unsafe {
            leptonica_sys::boxaDestroy(&mut self.raw);
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
            let b =
                leptonica_sys::boxaGetBox(self.raw, index as i32, leptonica_sys::L_CLONE as i32);
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
