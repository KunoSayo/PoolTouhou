use std::collections::HashMap;
use std::io::{Read, Write};
use std::str::FromStr;

pub struct Config {
    path: String,
    map: HashMap<String, String>,
    lines: Vec<LineType>,
    dirty: bool,
}

enum LineType {
    KeyValue(String),
    Line(String),
}

impl Config {
    pub fn read_from_path(path: &str) -> std::io::Result<Self> {
        let mut s = String::new();
        let mut file = std::fs::OpenOptions::new().write(true).read(true).create(true).open(path)?;
        file.read_to_string(&mut s)?;
        let mut lines = Vec::new();
        let mut map = HashMap::new();
        for x in s.lines() {
            let mut v = x.splitn(2, "=");
            if let Some(k) = v.next() {
                lines.push(LineType::KeyValue(k.into()));
                if let Some(v) = v.next() {
                    map.insert(k.into(), v.into());
                } else {
                    map.insert(k.into(), String::new());
                }
            } else {
                lines.push(LineType::Line(x.into()));
            }
        }
        if let Some(LineType::Line(l)) = lines.last() {
            if l.is_empty() {
                lines.pop().expect("Checked it is last.");
            }
        }
        Ok(Self {
            path: path.into(),
            map,
            lines,
            dirty: false,
        })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    pub fn parse_or_default<T: FromStr>(&mut self, key: &str, default: &str) -> T
        where <T as FromStr>::Err: std::fmt::Debug
    {
        let map = &mut self.map;
        let entry = map.entry(key.into());
        let lines = &mut self.lines;
        let dirty = &mut self.dirty;
        entry.or_insert_with(|| {
            *dirty = true;
            lines.push(LineType::KeyValue(key.into()));
            default.into()
        }).parse().unwrap_or_else(|_| {
            *dirty = true;
            map.insert(key.into(), default.into());
            default.parse().expect("Even the default value cannot be parsed")
        })
    }

    pub fn or_default(&mut self, key: &str, default: &str) -> &String {
        let entry = self.map.entry(key.into());
        let lines = &mut self.lines;
        let dirty = &mut self.dirty;
        entry.or_insert_with(|| {
            *dirty = true;
            lines.push(LineType::KeyValue(key.into()));
            default.into()
        })
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(v) = self.map.insert(key.into(), value.into()) {
            if v != value {
                self.dirty = true;
            }
        } else {
            self.lines.push(LineType::KeyValue(key.into()));
        }
    }

    pub fn save(&mut self) -> std::io::Result<bool> {
        if self.dirty {
            // log::info!("Saving pool touhou config");
            let mut file = std::fs::OpenOptions::new().write(true).truncate(true).read(true).create(true).open(&self.path)?;
            for x in &self.lines {
                match x {
                    LineType::KeyValue(k) => {
                        let line = format!("{}={}", k, self.map.get(k).map(|x| x as &str).unwrap_or(""));
                        file.write_all(line.as_bytes())?;
                        file.write_all(b"\n")?;
                    }
                    LineType::Line(l) => {
                        file.write_all(l.as_bytes())?;
                        file.write_all(b"\n")?;
                    }
                }
            }
            self.dirty = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}