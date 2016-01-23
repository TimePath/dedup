extern crate argparse;
extern crate crypto;
extern crate multimap;
extern crate walkdir;

use argparse::{ArgumentParser, Store};

use crypto::digest::Digest;
use crypto::sha1::Sha1;

use multimap::MultiMap;

use walkdir::WalkDir;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;

fn main() {
	let mut dir = String::new();
	let mut tmp = "/tmp/hash".to_string();
	{
		let mut ap = ArgumentParser::new();
		ap.refer(&mut dir)
			.add_argument("target", Store, "Target directory")
			.required()
			;
		ap.refer(&mut tmp)
			.add_option(&["-t", "--tmp"], Store, "Working directory")
			;
		ap.parse_args_or_exit();
	}
	let mut map = MultiMap::new();
	for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
		let path = entry.path().to_path_buf();
		if path.is_dir() { continue; }
		let f = match File::open(entry.path()) {
			Err(_) => continue,
			Ok(it) => it
		};
		let mut reader = BufReader::new(f);
		let mut hasher = Sha1::new();
		loop {
			let n = {
				let bytes = reader.fill_buf().unwrap();
				hasher.input(bytes);
				bytes.len()
			};
			if n == 0 { break; }
			reader.consume(n);
		}
		let hash = hasher.result_str();
		let s = {
			let s = path.as_path().to_str().unwrap();
			String::from_str(s).unwrap()
		};
		println!("# {} {}", hash, entry.path().display());
		map.insert(hash, s);
	}
	println!("set -eu");
	println!("OUT={}", tmp);
	println!("mkdir -p \"$OUT\"");
	println!("_log() {{ echo \"$@\"; }}");
	println!("_dup() {{ cp -l \"$1\" \"$OUT/$HASH\"; }}");
	println!("_ref() {{ cmp \"$OUT/$HASH\" \"$1\" && ln -b -f \"$OUT/$HASH\" \"$1\"; }}");
	let total = {
		let mut total = 0;
		for (_, files) in map.iter_all() {
			if files.len() == 1 { continue; }
			total += files.len();
		}
		total
	};
	let mut i = 0;
	for (id, files) in map.iter_all() {
		if files.len() == 1 { continue; }
		i += files.len();
		println!("\n_log {}/{}", i, total);
		println!("HASH={}", id);
		fn esc(s: &str) -> String { s.replace("\\", "\\\\").replace("\"", "\\\"").replace("$", "\\$").replace("`", "\\`") }
		println!("_dup \"{}\"", esc(&files[0]));
		for file in files { println!("_ref \"{}\"", esc(file)); }
	}
}
