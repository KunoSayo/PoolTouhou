use std::collections::HashMap;
use std::io::{Read, Write};

pub struct Config {
    path: String,
    map: HashMap<String, String>,
    dirty: bool,
}

impl Config {
    pub fn read_from_path(path: &str) -> std::io::Result<Self> {
        let mut s = String::new();
        let mut file = std::fs::OpenOptions::new().write(true).read(true).create(true).open(path)?;
        file.read_to_string(&mut s)?;

        let mut map = HashMap::new();
        for x in s.lines() {
            let mut v = x.splitn(2, ".");
            if let (Some(k), Some(v)) = (v.next(), v.next()) {
                map.insert(k.into(), v.into());
            }
        }
        Ok(Self {
            path: path.into(),
            map,
            dirty: false,
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if self.dirty {
            let mut file = std::fs::File::open(&self.path)?;
            for x in &self.map {
                let line = format!("{}={}", x.0, x.1);
                file.write_all(line.as_bytes())?;
                file.write(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }
}