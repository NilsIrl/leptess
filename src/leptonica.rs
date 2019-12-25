//! Low level wrapper for Leptonica C API

use super::capi;

use std::ffi::CString;
use std::path::Path;

pub struct Pix {
    pub raw: *mut capi::Pix,
}

impl Pix {
    pub fn get_w(&self) -> u32 {
        unsafe { (*self.raw).w }
    }
    pub fn get_h(&self) -> u32 {
        unsafe { (*self.raw).h }
    }
}

impl Drop for Pix {
    fn drop(&mut self) {
        unsafe {
            capi::pixDestroy(&mut self.raw);
        }
    }
}

pub fn pix_read(path: &Path) -> Option<Pix> {
    let s = path.to_str().unwrap();

    unsafe {
        let pix = capi::pixRead(CString::new(s).unwrap().as_ptr());
        if pix.is_null() {
            return None;
        }

        return Some(Pix { raw: pix });
    }
}

#[derive(Debug, PartialEq)]
pub struct BoxVal {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
    pub fn get_val(&self) -> BoxVal {
        unsafe {
            let v = *self.raw;
            BoxVal {
                x: v.x,
                y: v.y,
                w: v.w,
                h: v.h,
            }
        }
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
