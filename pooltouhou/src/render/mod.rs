pub mod texture2d;
// pub use invert_color::InvertColorCircle;
// pub use invert_color::RenderInvertColorCircle;
// use glsl_layout::*;
//
// use std::path::PathBuf;
//
// pub mod blit;
// pub mod invert_color;
// pub mod water_wave;
//
// #[derive(Clone, Copy, Debug, AsStd140)]
// #[repr(C, align(4))]
// pub struct PthCameraUniformArgs {
//     pub projection: mat4,
//     pub view: mat4,
// }


//lazy_static::lazy_static! {
//     pub static ref BLIT_VERTEX: SpirvShader = PathBufShaderInfo::new(
//         PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/blit.vert"),
//         ShaderKind::Vertex,
//         SourceLanguage::GLSL,
//        "main",
//     ).precompile().unwrap();
//
//     pub static ref BLIT_FRAG: SpirvShader = PathBufShaderInfo::new(
//         PathBuf::from(std::env::current_dir().unwrap().to_str().unwrap().to_owned() + "/assets/shaders/blit.frag"),
//         ShaderKind::Fragment,
//         SourceLanguage::GLSL,
//         "main",
//     ).precompile().unwrap();
// }

