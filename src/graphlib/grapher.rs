use adw::gtk::gdk_pixbuf::Pixbuf;
use adw::gtk::gdk_pixbuf::ffi::GdkPixbuf;
use adw::glib::translate::FromGlibPtrFull;
use std::ffi::CString;

unsafe extern "C" {
    fn set_expression(expr: *const std::os::raw::c_char);
    fn create_pixbuf(
        width: i32,
        height: i32,
        rightx: f64,
        leftx: f64,
        topy: f64,
        bottomy: f64,
    ) -> *mut GdkPixbuf;
}


pub unsafe fn create_graph_pixbuf(width: i32, height: i32, scale: i32, expr: &str) -> Option<Pixbuf> {
    let c_expr = CString::new(expr).ok()?;
    unsafe { set_expression(c_expr.as_ptr()) };
    let sizex: f64 = (width/scale/2).into();
    let sizey: f64 = (height/scale/2).into();
    let raw = unsafe { create_pixbuf(width, height, sizex, -sizex, sizey, -sizey) };
    if raw.is_null() {
        None
    } else {
        Some(unsafe { FromGlibPtrFull::from_glib_full(raw) })
    }
}