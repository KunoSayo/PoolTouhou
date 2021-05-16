use std::collections::HashMap;

use std::path::PathBuf;
use wgpu_glyph::ab_glyph::{FontVec, FontArc};
use std::sync::Arc;

pub struct ResourcesHandles {
    pub res_root: PathBuf,
    assets_dir: PathBuf,
    pub fonts: HashMap<String, FontArc>,
}

impl Default for ResourcesHandles {
    fn default() -> Self {
        let app_root = std::env::current_dir().unwrap();
        let res_root = if app_root.join("res").exists() { app_root.join("res") } else { app_root };
        let assets_dir = res_root.join("assets");
        Self {
            res_root,
            assets_dir,
            fonts: Default::default(),
        }
    }
}

impl ResourcesHandles {
    pub fn load_font(&mut self, name: &str, file_name: &str) {
        let target = self.assets_dir.join("font").join(file_name);
        let font_arc = wgpu_glyph::ab_glyph::FontArc::try_from_vec(
            std::fs::read(target)
                .expect("read font file failed")).unwrap();
        self.fonts.insert(name.to_string(), font_arc);
    }
}