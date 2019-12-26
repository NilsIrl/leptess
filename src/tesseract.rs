//! Low level wrapper for Tesseract C API

use super::capi;
use super::leptonica;

#[derive(Debug, PartialEq)]
pub struct TessInitError {
    pub code: i32,
}

impl std::fmt::Display for TessInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TessInitError{{{}}}", self.code)
    }
}

struct TessBaseApiUninitializedPointer {
    raw: *mut capi::TessBaseAPI,
}

struct TessBaseApiInitializedPointer {
    raw: *mut capi::TessBaseAPI,
}

impl TessBaseApiUninitializedPointer {
    fn new() -> TessBaseApiUninitializedPointer {
        TessBaseApiUninitializedPointer {
            raw: unsafe { capi::TessBaseAPICreate() },
        }
    }
}

impl Drop for TessBaseApiUninitializedPointer {
    fn drop(&mut self) {
        unsafe {
            capi::TessBaseAPIDelete(self.raw);
        }
    }
}

impl Drop for TessBaseApiInitializedPointer {
    fn drop(&mut self) {
        unsafe {
            capi::TessBaseAPIEnd(self.raw);
            capi::TessBaseAPIDelete(self.raw);
        }
    }
}

pub struct TessBaseApiUnitialized {
    pointer: TessBaseApiUninitializedPointer,
}

pub struct TessBaseApiInitialized {
    pointer: TessBaseApiInitializedPointer,
}

pub struct TessBaseApiImageSet {
    pointer: TessBaseApiInitializedPointer,
}

impl TessBaseApiUnitialized {
    pub fn new() -> TessBaseApiUnitialized {
        TessBaseApiUnitialized {
            pointer: TessBaseApiUninitializedPointer::new(),
        }
    }

    pub fn init(self) -> TessBaseApiInitialized {
        unsafe {
            capi::TessBaseAPIInit3(self.pointer.raw, std::ptr::null(), std::ptr::null());
        }
        Self::create_tess_base_api_initialized(self)
    }

    pub fn init_with_lang(self, language: &str) -> TessBaseApiInitialized {
        unsafe {
            capi::TessBaseAPIInit3(
                self.pointer.raw,
                std::ptr::null(),
                std::ffi::CString::new(language).unwrap().as_ptr(),
            );
        }
        Self::create_tess_base_api_initialized(self)
    }

    pub fn init_with_datapath(self, datapath: std::path::Path) -> TessBaseApiInitialized {
        unsafe {
            capi::TessBaseAPIInit3(
                self.pointer.raw,
                std::ffi::CString::new(datapath.to_str().unwrap()).as_ptr(),
                std::ptr::null(),
            );
        }
        Self::create_tess_base_api_initialized(self)
    }

    fn create_tess_base_api_initialized(
        tess_base_api_uninitialized: TessBaseApiUnitialized,
    ) -> TessBaseApiInitialized {
        let tess_base_api_initialized = TessBaseApiInitialized {
            pointer: TessBaseApiInitializedPointer {
                raw: tess_base_api_uninitialized.pointer.raw,
            },
        };
        std::mem::forget(tess_base_api_uninitialized);
        tess_base_api_initialized
    }
}

impl TessBaseApiInitialized {
    /// Drops self and returns TessBaseApiImageSet signifying an image has been given
    pub fn set_image(self, img: &leptonica::Pix) -> TessBaseApiImageSet {
        unsafe { capi::TessBaseAPISetImage2(self.pointer.raw, img.raw) }
        let tess_api_image_set = TessBaseApiImageSet {
            pointer: TessBaseApiInitializedPointer {
                raw: self.pointer.raw,
            },
        };
        std::mem::forget(self);
        tess_api_image_set
    }
}

pub enum PageIteratorLevel {
    Block,
    Para,
    Textline,
    Word,
    Symbol,
}

impl TessBaseApiImageSet {
    pub fn set_rectangle(&self, rectangle: leptonica::Box) {
        unsafe {
            capi::TessBaseAPISetRectangle(
                self.pointer.raw,
                rectangle.x(),
                rectangle.y(),
                rectangle.w(),
                rectangle.h(),
            )
        }
    }

    pub fn get_text(&self) -> String {
        unsafe {
            let sptr = capi::TessBaseAPIGetUTF8Text(self.pointer.raw);
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
                self.pointer.raw,
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
