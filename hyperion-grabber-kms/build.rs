use std::env;
use std::fs::File;
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn main() {
    // XXX this is taken from glutin/build.rs.

    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    let mut file = File::create(dest.join("gl_bindings.rs")).unwrap();
    Registry::new(
        Api::Gles2,
        (3, 0),
        Profile::Core,
        Fallbacks::All,
        [
            "GL_OES_EGL_image",
            "GL_OES_EGL_image_external",
            "GL_KHR_debug",
        ],
    )
    .write_bindings(StructGenerator, &mut file)
    .unwrap();

    let mut file = File::create(dest.join("egl_bindings.rs")).unwrap();
    Registry::new(
        Api::Egl,
        (1, 5),
        Profile::Core,
        Fallbacks::All,
        [
            "EGL_EXT_image_dma_buf_import",
            "EGL_EXT_image_dma_buf_import_modifiers",
        ],
    )
    .write_bindings(StructGenerator, &mut file)
    .unwrap();
}
