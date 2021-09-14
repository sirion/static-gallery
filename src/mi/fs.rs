use std::path::PathBuf;
use crate::logger::{errorln, debugln};


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

pub fn list_dir(dir: &PathBuf) -> Vec<PathBuf> {
	let mut files: Vec<PathBuf> = Vec::new();

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
		match file {
			Ok(f) => {
				files.push(f.path());
			},
			Err(e) => {
				errorln(format!("Could not read file: {}", e));
				continue;
			},
		};
	}

	files

}

pub fn list_dir_as_strings(dir: &PathBuf) -> Vec<String> {
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

	debugln(format!("Sanitized: \"{}\" => \"{}\"", s, cleaned));
	cleaned
}

pub fn hash_quick(data: Vec<u8>) -> u64 {
	let mut n = 0u64;

	for c in data {
		n += c as u64;
	}

	n
}


pub fn file_hash_quick(p: &PathBuf) -> u64 {
	hash_quick(std::fs::read(p).unwrap())
}
