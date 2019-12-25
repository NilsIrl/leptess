//! Low level wrapper for Tesseract C API

use super::capi;
use super::leptonica;

use std::ffi::CStr;
use std::ptr;

pub use capi::kMaxCredibleResolution as MAX_CREDIBLE_RESOLUTION;
pub use capi::kMinCredibleResolution as MIN_CREDIBLE_RESOLUTION;

#[derive(Debug, PartialEq)]
pub struct TessInitError {
    pub code: i32,
}

impl std::fmt::Display for TessInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TessInitError{{{}}}", self.code)
    }
}

#[derive(Debug, PartialEq)]
pub struct TessApi {
    pub raw: *mut capi::TessBaseAPI,
}

pub struct TessApiImageSet {
    pub raw: *mut capi::TessBaseAPI,
}

impl Drop for TessApi {
    fn drop(&mut self) {
        unsafe {
            capi::TessBaseAPIEnd(self.raw);
            capi::TessBaseAPIDelete(self.raw);
        }
    }
}

impl Drop for TessApiImageSet {
    fn drop(&mut self) {
        unsafe {
            capi::TessBaseAPIEnd(self.raw);
            capi::TessBaseAPIDelete(self.raw);
        }
    }
}

impl TessApi {
    pub fn new() -> Result<TessApi, TessInitError> {
        let api = TessApi {
            raw: unsafe { capi::TessBaseAPICreate() },
        };

        unsafe {
            let re = capi::TessBaseAPIInit3(api.raw, ptr::null(), ptr::null());

            if re == 0 {
                // TODO: https://github.com/tesseract-ocr/tesseract/issues/2832
                return Ok(api);
            } else {
                return Err(TessInitError { code: re });
            }
        }
    }

    pub fn set_image(&self, img: &leptonica::Pix) -> TessApiImageSet {
        unsafe { capi::TessBaseAPISetImage2(self.raw, img.raw) }
        TessApiImageSet { raw: self.raw }
    }

    pub fn get_source_y_resolution(&mut self) -> i32 {
        unsafe { capi::TessBaseAPIGetSourceYResolution(self.raw) }
    }

    /// Override image resolution.
    /// Can be used to suppress "Warning: Invalid resolution 0 dpi." output.
    pub fn set_source_resolution(&mut self, res: i32) {
        unsafe { capi::TessBaseAPISetSourceResolution(self.raw, res) }
    }

    pub fn recognize(&self) -> i32 {
        unsafe { capi::TessBaseAPIRecognize(self.raw, ptr::null_mut()) }
    }

    pub fn mean_text_conf(&self) -> i32 {
        unsafe { capi::TessBaseAPIMeanTextConf(self.raw) }
    }

    pub fn get_regions(&self) -> Option<leptonica::Boxes> {
        unsafe {
            let boxes = capi::TessBaseAPIGetRegions(self.raw, ptr::null_mut());
            if boxes.is_null() {
                return None;
            }
            return Some(leptonica::Boxes { raw: boxes });
        }
    }
}

pub enum PageIteratorLevel {
    Block,
    Para,
    Textline,
    Word,
    Symbol,
}

impl TessApiImageSet {
    pub fn set_rectangle(&self, rectangle: leptonica::Box) {
        unsafe {
            capi::TessBaseAPISetRectangle(
                self.raw,
                rectangle.x(),
                rectangle.y(),
                rectangle.w(),
                rectangle.h(),
            )
        }
    }

    pub fn get_text(&self) -> String {
        unsafe {
            let sptr = capi::TessBaseAPIGetUTF8Text(self.raw);
            let re = CStr::from_ptr(sptr).to_str().unwrap().to_string();
            capi::TessDeleteText(sptr);
            return re;
        }
    }

    /// Get the given level kind of components (block, textline, word etc.) as a leptonica-style
    /// Boxa, in reading order. If text_only is true, then only text components are returned.
    /// https://tesseract-ocr.github.io/4.0.0/a01625.html#gad74ae1266a5299734ec6f5225b6cb5c1
    pub fn get_component_images(
        &self,
        level: PageIteratorLevel,
        text_only: bool,
    ) -> leptonica::Boxes {
        unsafe {
            let boxes = capi::TessBaseAPIGetComponentImages(
                self.raw,
                match level {
                    PageIteratorLevel::Block => capi::TessPageIteratorLevel_RIL_BLOCK,
                    PageIteratorLevel::Para => capi::TessPageIteratorLevel_RIL_PARA,
                    PageIteratorLevel::Textline => capi::TessPageIteratorLevel_RIL_TEXTLINE,
                    PageIteratorLevel::Word => capi::TessPageIteratorLevel_RIL_WORD,
                    PageIteratorLevel::Symbol => capi::TessPageIteratorLevel_RIL_SYMBOL,
                },
                if text_only { 1 } else { 0 },
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if boxes.is_null() {
                unreachable!();
            }
            return leptonica::Boxes { raw: boxes };
        }
    }
}
