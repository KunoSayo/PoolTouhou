use std::ffi::OsString;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

mod game_data;

mod context;
mod expression;

pub mod pool_script;

fn compile(name: OsString, file: File) {
    println!("compiling file {:?}", name);

    let script = pool_script::PoolScript::try_parse(Box::new(BufReader::new(file)));
    if script.is_err() {
        eprintln!("parse script failed: {}", script.err().expect("parse failed: Unknown"));
        return;
    }
    let output_name = OsString::from(name.to_str().unwrap().to_owned() + "b");
    let output_file = File::create(&output_name);
    if output_file.is_err() {
        eprintln!("open output file failed: {}", script.err().expect("save failed: Unknown"));
        return;
    }

    let result = script.unwrap().save(&mut BufWriter::new(output_file.unwrap()));
    if result.is_err() {
        eprintln!("save output file failed: {}", result.err().expect("save failed: Unknown"));
    } else {
        println!("compiled file {:?} into {:?}", name, output_name);
    }
}

//https://doc.rust-lang.org/book/

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("psc compile <dir...>")
    } else if args.len() > 2 && args[1] == "compile" {
        let run_dir = std::env::current_dir().unwrap();
        for path in args.iter().skip(2) {
            let dir_path = run_dir.join(Path::new(path));
            let dir = dir_path.read_dir().expect(&*("We need a directory, not ".to_owned() + dir_path.to_str().unwrap()));
            println!("compiling dir {:?}", dir);
            for file in dir {
                match file {
                    Ok(entry) => {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() && entry.file_name().to_str().to_owned().unwrap().ends_with(".pthps") {
                                match File::open(entry.path()) {
                                    Ok(file) => compile(entry.file_name(), file),
                                    Err(err) => eprintln!("open file failed: {}", err)
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("read entry failed! {}", err);
                    }
                }
            }
        }
    }
}
