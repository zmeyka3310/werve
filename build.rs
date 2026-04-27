fn main() {
    let gdk_pixbuf = pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("gdk-pixbuf-2.0")
        .expect("gdk-pixbuf-2.0 not found");

    cc::Build::new()
        .file("src/graphlib/pixbuf.c")
        .includes(&gdk_pixbuf.include_paths)
        .flag("-Wno-unused-parameter")
        .compile("graphlib_pixbuf");
}