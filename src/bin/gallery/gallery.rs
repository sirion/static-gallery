use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use threadpool::ThreadPool;
use zip::write::FileOptions;
use mi::logger::{info, infoln, errorln};
use crate::mi::bin::Replace;
use crate::mi::img::Resolution;
use crate::gallery::Collection;
use crate::gallery::CollectionInput;
use crate::gallery::Image;



#[derive(Debug, Serialize, Deserialize)]
pub struct Gallery {
	pub version: u16,
	pub extension: String,
	pub archives: HashMap<String, String>,
	pub collection_keys: Vec<String>,
	pub collections: HashMap<String, Collection>,

	pub res_background: Resolution,
	pub res_display: Resolution,
	pub res_thumb: Resolution,
}

impl Gallery {
	pub fn new() -> Gallery {
		Gallery{
			version: crate::gallery::GALLERY_CONFIGURATION_VERSION,
			extension: String::from(crate::gallery::PICTURE_EXTENSION),
			archives: HashMap::new(),
			collection_keys: Vec::new(),
			collections: HashMap::new(),
			res_background: crate::gallery::DEFAULT_RESOLUTION_BACKGROUND,
			res_display: crate::gallery::DEFAULT_RESOLUTION_DISPLAY,
			res_thumb: crate::gallery::DEFAULT_RESOLUTION_THUMB
		}
	}

	pub fn from(gallery_dir: &PathBuf)  -> Gallery {
		// TODO: You cannot override resolutions when updating

		// TODO: Get json from index.html and deserialize
		let index_path = gallery_dir.join("index.html");
		let data = match std::fs::read(index_path) {
			Ok(d) => d,
			Err(e) => {
				errorln(format!("Cannot update existing gallery. Error: {}", e));
				std::process::exit(1); // TODO: Consistent exit codes
			}
		};

		let json = data.between(crate::gallery::PATTERM_DATA_START, crate::gallery::PATTERM_DATA_END);

		let gallery = match serde_json::from_slice(&json) {
			Ok(g) => g,
			Err(e) => {
				errorln(format!("Cannot update existing gallery. Invalid Data. Error: {}", e));
				std::process::exit(2); // TODO: Consistent exit codes
			}
		};

		gallery
	}


	pub fn fill(&mut self, collection_inputs: Vec<CollectionInput>, use_filenames_as_titles: bool) -> Result<(), String> {
		for mut c in collection_inputs {
			c.exists = self.collections.contains_key(&c.name);


			let background_paths: Vec<PathBuf> = match &c.background_dir {
				Some(d) => crate::mi::fs::list_dir(&d),
				None => Vec::new(),
			};

			let picture_paths: Vec<PathBuf> = match &c.input_dir {
				Some(d) => crate::mi::fs::list_dir(&d),
				None => Vec::new(),
			};

			let collection = Collection::new(c.name.clone(), c.title.clone(), picture_paths, background_paths, use_filenames_as_titles);

			if c.exists {
				self.collections.get_mut(&c.name).unwrap().append(collection);
			} else {
				if c.input_dir.is_none() {
					return Err(format!("Cannot create new collection without input directory: {}", c.title));
				}
				self.collection_keys.push(c.name.clone());
				self.collections.insert(c.name, collection);
			}
		}

		Ok(())
	}

	pub fn remove_duplicates(&mut self) {
		// Remove duplicate pictures across galleries
		infoln(format!("Searching for duplicates... "));

		let mut file_hashes: HashMap<u64, Vec<Image>> = HashMap::new();

		for k in &self.collection_keys {

			for bg in &self.collections[k].backgrounds {
				let hash = bg.original_hash;

				if !file_hashes.contains_key(&hash) {
					file_hashes.insert(hash, vec![]);
				}
				let basenames = file_hashes.get_mut(&hash).unwrap();
				basenames.push(bg.clone());
			}

			for pic in &self.collections[k].pictures {
				let hash = pic.image.original_hash;

				if !file_hashes.contains_key(&hash) {
					file_hashes.insert(hash, vec![]);
				}
				let basenames = file_hashes.get_mut(&hash).unwrap();
				basenames.push(pic.image.clone());
			}
		}

		for (_, files) in file_hashes {
			let num_files = files.len();
			assert_ne!(num_files, 0);
			if num_files < 2 {
				continue;
			}

			// Duplicate found
			for (i, file) in files.iter().enumerate() {
				// TODO: Replace all but the first with the first
				if i == 0 {
					continue;
				}
				self.replace_image(file.basename.clone(), &files[0].clone());
			}

		}

	}

	pub fn replace_image(&mut self, basename: String, with: &Image) {
		infoln(format!("Replacing {} with {}", basename, with.basename));

		// Make sure the first updatable match is not replaced to keep the update property value
		let mut rep = 0;

		for c in self.collections.values_mut() {
			for img in c.backgrounds.iter_mut() {
				if img.basename == basename {
					rep += 1;
					if img.update && rep == 1 {
						continue;
					}

					img.basename = with.basename.clone();
					img.source_path = with.source_path.clone();
					img.original_hash = with.original_hash;
					img.update = false;

				}
			}

			for pic in c.pictures.iter_mut() {
				if pic.image.basename == basename {
					rep += 1;
					if rep == 1 {
						continue;
					}

					pic.image.basename = with.basename.clone();
					pic.image.source_path = with.source_path.clone();
					pic.image.original_hash = with.original_hash;
					pic.image.update = false;
				}
			}
		}

	}


	pub fn create_images(&mut self, output_dir: &PathBuf, quality: u8, method: &String, num_threads: usize) -> Result<(), Box<dyn std::error::Error>> {
		let pool = ThreadPool::new(num_threads);

		for (_, c) in self.collections.iter_mut() {
			c.create_images(&pool, output_dir, quality, method, self.res_thumb, self.res_display, self.res_background)?;
		}

		info(format!("Working on pictures... {} left.\r", pool.queued_count() + pool.active_count()));
		let mut i = 0;
		while pool.queued_count() + pool.active_count() > 0 {
			i = i + 1;
			let n = i % 4;
			info(format!("Working on pictures{}{} {} left. \r", ".".repeat(n), " ".repeat(3 - n), pool.queued_count() + pool.active_count()));
			std::thread::sleep(std::time::Duration::from_millis(500));
		}
		infoln(format!("Working on pictures. Done         "));
		pool.join();

		Ok(())
	}


	pub fn create_archive_full(&mut self, output_dir: &PathBuf) {
		let archive_path = std::path::PathBuf::from(output_dir).join(crate::gallery::FULL_ARCHIVE_PATH);
		self.archives.insert(String::from("_full_"), String::from(crate::gallery::FULL_ARCHIVE_PATH));

		let archive_file = std::fs::File::create(archive_path).unwrap();
		let mut zip = zip::ZipWriter::new(archive_file);
		let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		for (_, c) in &self.collections {
			let dir_name = crate::mi::fs::sanitize(c.title.as_str());
			zip.add_directory(&dir_name, options.clone()).unwrap();
			// println!("Starting dir {}", dir_name);
			for p in &c.pictures {
				let path = p.image.source_path.clone();
				let pic = path.file_name().unwrap().to_str().unwrap();
				// println!("Starting file {}", pic);
				let file_path = format!("{}/{}", &dir_name, pic);
				zip.start_file(file_path, options).unwrap();
				let mut buf = std::fs::read(p.image.source_path.clone()).unwrap();
				zip.write_all(&mut buf).unwrap();
			}
		}
		zip.finish().unwrap();

	}

	pub fn include_json_data(&self, output_dir: &PathBuf) {
		let json = serde_json::to_string_pretty(self).unwrap();

		// Insert JSON data in the index.html
		let index_path = std::path::PathBuf::from(output_dir).join("index.html");
		let html = match std::fs::read(&index_path) {
			Ok(d) => d,
			Err(e) => {
				panic!("Could not read from {}: {}", &index_path.to_str().unwrap(), e);
			}
		};

		let replaced_html = html.replace_between(crate::gallery::PATTERM_DATA_START, crate::gallery::PATTERM_DATA_END, json.as_bytes());

		match std::fs::write(&index_path, replaced_html) {
			Ok(_) => {},
			Err(e) => {
				panic!("Could not write to {}: {}", &index_path.to_str().unwrap(), e);
			}
		}
	}

}
