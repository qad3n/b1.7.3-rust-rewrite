use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
use std::env;
use std::fs::File;
use std::path::PathBuf;

// the game draws with the fixed function pipeline of an OpenGL 2.1 compatibility context
fn main()
{
    let out: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let mut file: File = File::create(out.join("gl_bindings.rs")).expect("OUT_DIR is writable");
    let extensions: [&str; 3] = ["GL_ARB_occlusion_query", "GL_ARB_vertex_buffer_object", "GL_NV_fog_distance"];
    Registry::new(Api::Gl, (2, 1), Profile::Compatibility, Fallbacks::All, extensions)
        .write_bindings(StructGenerator, &mut file)
        .expect("gl bindings generate");
}
