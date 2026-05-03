fn main() {
    let gdk_pixbuf = pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("gdk-pixbuf-2.0")
        .expect("gdk-pixbuf-2.0 not found");

    cc::Build::new()
        .file("src/graphlib/pixbuf.c")
        .includes(&gdk_pixbuf.include_paths)
        .opt_level(0) // this is needed to not break jit in graphlib
        .compile("pixbuf");
    println!("cargo:rerun-if-changed=src/graphlib/pixbuf.c");
    println!("cargo:rustc-link-lib=tcc");
    println!("cargo:rustc-link-lib=m");
}