//! Web Utilities for Dreg



#[inline(always)]
pub fn window() -> web_sys::Window {
    web_sys::window()
        .expect("should have a window object")
}

#[inline(always)]
pub fn document(window: &web_sys::Window) -> web_sys::Document {
    window.document()
        .expect("window should have a document object")
}
