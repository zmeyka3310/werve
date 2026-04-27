use adw::gtk::gdk_pixbuf::Pixbuf;
use adw::gtk::gdk_pixbuf::ffi::GdkPixbuf;
use adw::glib::translate::FromGlibPtrFull;

unsafe extern "C" {
    fn create_pixbuf(
        width: i32,
        height: i32,
        rightx: f64,
        leftx: f64,
        topy: f64,
        bottomy: f64,
    ) -> *mut GdkPixbuf;
}


pub unsafe fn create_blank_pixbuf(width: i32, height: i32) -> Option<Pixbuf> {
    let raw = unsafe { create_pixbuf(width, height, 0.0, 0.0, 0.0, 0.0) };
    if raw.is_null() {
        None
    } else {
        Some(unsafe { FromGlibPtrFull::from_glib_full(raw) })
    }
}