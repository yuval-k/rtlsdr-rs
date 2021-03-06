extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    // Tell cargo to tell rustc to link the system rtlsdr
    // shared library.
     #[cfg(not(feature = "static"))]
        println!("cargo:rustc-link-lib=rtlsdr");

     #[cfg(feature = "static")] 
     {
        println!("cargo:rustc-link-lib=static=rtlsdr");
        // TODO: re enable this if\when we build the library from source
        // println!("cargo:rustc-link-search=native={}", out_dir);
     }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
