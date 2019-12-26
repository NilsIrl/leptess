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

    // Return a result instead of panicking if -1 is reachable
    fn init(&self, datapath: *const i8, language: *const i8) {
        match unsafe { capi::TessBaseAPIInit3(self.raw, datapath, language) } {
            0 => (),
            -1 => panic!("Failed to initialize"),
            _ => unreachable!(),
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
        self.pointer.init(std::ptr::null(), std::ptr::null());
        self.create_tess_base_api_initialized()
    }

    pub fn init_with_lang(self, language: &str) -> TessBaseApiInitialized {
        self.pointer.init(
            std::ptr::null(),
            std::ffi::CString::new(language).unwrap().as_ptr(),
        );
        self.create_tess_base_api_initialized()
    }

    pub fn init_with_datapath(self, datapath: &std::path::Path) -> TessBaseApiInitialized {
        unsafe {
            capi::TessBaseAPIInit3(
                self.pointer.raw,
                std::ffi::CString::new(datapath.to_str().unwrap())
                    .unwrap()
                    .as_ptr(),
                std::ptr::null(),
            );
        }
        self.create_tess_base_api_initialized()
    }

    pub fn init_with_datapath_and_lang(
        self,
        datapath: &std::path::Path,
        language: &str,
    ) -> TessBaseApiInitialized {
        self.pointer.init(
            std::ffi::CString::new(datapath.to_str().unwrap())
                .unwrap()
                .as_ptr(),
            std::ffi::CString::new(language).unwrap().as_ptr(),
        );
        self.create_tess_base_api_initialized()
    }

    fn create_tess_base_api_initialized(self) -> TessBaseApiInitialized {
        let tess_base_api_initialized = TessBaseApiInitialized {
            pointer: TessBaseApiInitializedPointer {
                raw: self.pointer.raw,
            },
        };
        std::mem::forget(self);
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

impl TessBaseApiImageSet {
    pub fn set_rectangle(&self, rectangle: &leptonica::Box) {
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
            let re = std::ffi::CStr::from_ptr(sptr).to_str().unwrap().to_string();
            capi::TessDeleteText(sptr);
            return re;
        }
    }

    // Not public cause maybe not so idiomatic
    fn get_component_images(&self, iterator_level: u32, text_only: bool) -> leptonica::Boxes {
        leptonica::Boxes {
            raw: unsafe {
                capi::TessBaseAPIGetComponentImages(
                    self.pointer.raw,
                    iterator_level,
                    if text_only { 1 } else { 0 },
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
        }
    }

    pub fn get_blocks(&self, text_only: bool) -> leptonica::Boxes {
        self.get_component_images(capi::TessPageIteratorLevel_RIL_BLOCK, text_only)
    }

    pub fn get_paras(&self, text_only: bool) -> leptonica::Boxes {
        self.get_component_images(capi::TessPageIteratorLevel_RIL_PARA, text_only)
    }

    pub fn get_textlines(&self, text_only: bool) -> leptonica::Boxes {
        self.get_component_images(capi::TessPageIteratorLevel_RIL_TEXTLINE, text_only)
    }

    pub fn get_words(&self, text_only: bool) -> leptonica::Boxes {
        self.get_component_images(capi::TessPageIteratorLevel_RIL_WORD, text_only)
    }

    pub fn get_symbols(&self, text_only: bool) -> leptonica::Boxes {
        self.get_component_images(capi::TessPageIteratorLevel_RIL_SYMBOL, text_only)
    }
}
