use std::path::PathBuf;

pub fn dir_exists(dir: &PathBuf) -> bool {
	match std::fs::metadata(std::path::Path::new(&dir)) {
		Ok(m) => m.is_dir(),
		Err(_) => false,
	}
}

pub fn file_exists(dir: &PathBuf) -> bool {
	match std::fs::metadata(std::path::Path::new(&dir)) {
		Ok(m) => m.is_file(),
		Err(_) => false,
	}
}


pub fn copy_recursively(from: &PathBuf, to: &PathBuf) {
	if from.is_file() {
		std::fs::copy(from, to).unwrap();
	} else if from.is_dir() {
		let entries = match std::fs::read_dir(from) {
			Ok(f) => f,
			Err(e) => {
				panic!("{}", e);
			}
		};

		if !to.is_dir() {
			std::fs::create_dir_all(to).unwrap();
		}

		for entry in entries {
			let path = match entry {
				Ok(f) => f.path(),
				Err(_) => {
					continue;
				},
			};

			let mut to = to.clone();
			to.push(path.file_name().unwrap());

			copy_recursively(&path, &to);
		}

	} else {
		panic!("Unsupported type of directory entry: {}", from.to_str().unwrap());
	}
}

pub fn list_dir(dir: &PathBuf) -> Vec<String> {
	let mut files: Vec<String> = Vec::new();
	if !dir.is_dir() {
		return files;
	};


	// Validate input folders can be accessed
	let entries = match std::fs::read_dir(dir) {
		Ok(f) => f,
		Err(_) => {
			return files;
		}
	};

	for file in entries {
		// let mut path = dir.clone();
		let path = match file {
			Ok(f) => f.path(),
			Err(_) => {
				continue;
			},
		};

		files.push(String::from(path.to_str().unwrap()));
	}

	files
}

use crate::debugln;
pub fn sanitize(s: &str) -> String {
	let mut cleaned = String::new();

	for mut c in s.trim().chars() {
		c = c.to_ascii_lowercase();
		if c == '.' || c == '_' {
			// Keep c as is
		} else if c.is_whitespace() {
			c = '_';
		} else if !c.is_ascii_alphanumeric() {
			c = '-';
		}
		cleaned.push(c);
	}

	debugln!("Sanitized: \"{}\" => \"{}\"", s, cleaned);
	cleaned
}

pub fn clean_basename<T: AsRef<std::ffi::OsStr> + ToString>(path: &T) -> String {
	let p = std::path::Path::new(path);
	sanitize(p.with_extension("").file_name().unwrap().to_str().unwrap_or_default())
}

