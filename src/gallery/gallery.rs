use super::Collection;
use super::CollectionInput;
use mi::bin::Replace;
use mi::img::Resolution;
use mi::log_info;
use mi::logger::{errorln, info, infoln};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use zip::write::FileOptions;

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

impl Default for Gallery {
	fn default() -> Gallery {
		Gallery::new()
	}
}

impl Gallery {
	pub fn new() -> Gallery {
		Gallery {
			version: super::GALLERY_CONFIGURATION_VERSION,
			extension: String::from(super::PICTURE_EXTENSION),
			archives: HashMap::new(),
			collection_keys: Vec::new(),
			collections: HashMap::new(),
			res_background: super::DEFAULT_RESOLUTION_BACKGROUND,
			res_display: super::DEFAULT_RESOLUTION_DISPLAY,
			res_thumb: super::DEFAULT_RESOLUTION_THUMB,
		}
	}

	pub fn from(gallery_dir: &Path) -> Gallery {
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

		let json = data.between(super::PATTERM_DATA_START, super::PATTERM_DATA_END);

		let gallery = match serde_json::from_slice(&json) {
			Ok(g) => g,
			Err(e) => {
				errorln(format!(
					"Cannot update existing gallery. Invalid Data. Error: {}",
					e
				));
				std::process::exit(2); // TODO: Consistent exit codes
			}
		};

		gallery
	}

	pub fn fill(
		&mut self,
		collection_inputs: Vec<CollectionInput>,
		use_filenames_as_titles: bool,
	) -> Result<(), String> {
		for mut c in collection_inputs {
			c.exists = self.collections.contains_key(&c.name);

			let background_paths: Vec<PathBuf> = match &c.background_dir {
				Some(d) => mi::fs::list_dir(d),
				None => Vec::new(),
			};

			let picture_paths: Vec<PathBuf> = match &c.input_dir {
				Some(d) => mi::fs::list_dir(d),
				None => Vec::new(),
			};

			let collection = Collection::new(
				c.name.clone(),
				c.title.clone(),
				picture_paths,
				background_paths,
				use_filenames_as_titles,
			);

			if c.exists {
				self.collections
					.get_mut(&c.name)
					.unwrap()
					.append(collection);
			} else {
				if c.input_dir.is_none() {
					return Err(format!(
						"Cannot create new collection without input directory: {}",
						c.title
					));
				}
				self.collection_keys.push(c.name.clone());
				self.collections.insert(c.name, collection);
			}
		}

		Ok(())
	}

	pub fn create_images(
		&mut self,
		output_dir: &Path,
		quality: u8,
		method: &str,
		num_threads: usize,
	) -> Result<(), Box<dyn std::error::Error>> {
		let pool = ThreadPool::new(num_threads);

		let failed: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));

		for (_, c) in self.collections.iter_mut() {
			c.create_images(
				failed.clone(),
				&pool,
				output_dir,
				quality,
				method,
				self.res_thumb,
				self.res_display,
				self.res_background,
			)?;
		}

		let num_pics = pool.queued_count() + pool.active_count();
		info(format!("Working on pictures... ({} left).\r", num_pics));

		let mut i = 0;
		let mut num_left = num_pics;
		while num_left > 0 {
			i += 1;
			let n = i % 4;

			let percent = 100f32 * (1f32 - (num_left as f32 / num_pics as f32));
			info(format!(
				"Working on pictures{}{} {:>5.2}% ({} left). \r",
				".".repeat(n),
				" ".repeat(3 - n),
				percent,
				num_left
			));
			std::thread::sleep(std::time::Duration::from_millis(500));
			num_left = pool.queued_count() + pool.active_count();
		}
		infoln("Working on pictures. Done                ");
		pool.join();

		// All clones should have been dropped in the threads
		assert_eq!(Arc::strong_count(&failed), 1);
		// Now remove the images with errors
		let guard = failed.lock().unwrap();
		if guard.len() > 0 {
			log_info!("Removing {} invalid pictures from gallery", guard.len());
		}
		for (col_name, pic_name) in guard.iter() {
			let col: &mut Collection = self.collections.get_mut(col_name).unwrap();
			col.remove_resource(pic_name);
		}

		Ok(())
	}

	pub fn create_archive_full(&mut self, output_dir: &Path) {
		let archive_path = std::path::PathBuf::from(output_dir).join(super::FULL_ARCHIVE_PATH);
		self.archives.insert(
			String::from("_full_"),
			String::from(super::FULL_ARCHIVE_PATH),
		);

		let archive_file = std::fs::File::create(archive_path).unwrap();
		let mut zip = zip::ZipWriter::new(archive_file);
		let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		for c in self.collections.values() {
			let dir_name = mi::fs::sanitize(c.title.as_str());
			zip.add_directory(&dir_name, options).unwrap();
			// println!("Starting dir {}", dir_name);
			for p in &c.pictures {
				let path = p.image.source_path.clone();
				let pic = path.file_name().unwrap().to_str().unwrap();
				// println!("Starting file {}", pic);
				let file_path = format!("{}/{}", &dir_name, pic);
				zip.start_file(file_path, options).unwrap();
				let buf = std::fs::read(p.image.source_path.clone()).unwrap();
				zip.write_all(&buf).unwrap();
			}
		}
		zip.finish().unwrap();
	}

	pub fn include_json_data(&self, output_dir: &std::path::Path) {
		let json = serde_json::to_string_pretty(self).unwrap();

		// Insert JSON data in the index.html
		let index_path = std::path::PathBuf::from(output_dir).join("index.html");
		let html = match std::fs::read(&index_path) {
			Ok(d) => d,
			Err(e) => {
				panic!(
					"Could not read from {}: {}",
					&index_path.to_str().unwrap(),
					e
				);
			}
		};

		let replaced_html = html.replace_between(
			super::PATTERM_DATA_START,
			super::PATTERM_DATA_END,
			json.as_bytes(),
		);

		match std::fs::write(&index_path, replaced_html) {
			Ok(_) => {}
			Err(e) => {
				panic!(
					"Could not write to {}: {}",
					&index_path.to_str().unwrap(),
					e
				);
			}
		}
	}
}
