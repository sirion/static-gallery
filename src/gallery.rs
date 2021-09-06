use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use threadpool::ThreadPool;
use zip::write::FileOptions;
use crate::{info, infoln, debugln, warnln};
use crate::mi::bin::Replace;
use crate::mi::fs::clean_basename;
use crate::mi::img::Resolution;
use crate::mi::json::{Json, JsonType};


const FULL_ARCHIVE_PATH: &str = "Gallery.zip";
const PICTURE_EXTENSION: &str = "jpg";


fn valid_extension(path: &PathBuf) -> bool {
	let ext = match path.extension() {
		None => String::from(""),
		Some(e) => e.to_str().unwrap().to_lowercase(),
	};

	match ext.as_str() {
		"jpg" => true,
		"jpeg" => true,
		&_ => false,
	}
}

fn check_and_filter(images: &Vec<Picture>) -> Vec<Picture> {
	let mut filtered_images = vec![];
	for image in images {
		if valid_extension(&image.source_path) {
			filtered_images.push(image.clone());
		}
	}
	filtered_images
}

#[derive(Debug, Default)]
pub struct Gallery {
	pub extension: String,
	pub archives: HashMap<String, String>,
	pub collections: Vec<Collection>,
}

impl Gallery {
	pub fn from(config: &crate::Configuration) -> Result<Gallery, String> {
		let mut collections = Vec::new();

		let mut backgrounds = Vec::new();
		for dir in &config.background_dirs {
			let pictures = crate::mi::fs::list_dir(dir);
			backgrounds.push(pictures);
		}

		if backgrounds.len() == 0 {
			// No background pictures
			infoln!("Info: No backgrounds provided");
			backgrounds.push(vec![]);
		} else if backgrounds.len() == 1 {
			// Same background pictures for all collections
			infoln!("Info: Using same backgrounds for all collections");
		} else if backgrounds.len() > config.input_dirs.len() {
			// Weird, more than one background folder and more than one for each collection
			warnln!("Too many background directories given. Ignoring the last {}", backgrounds.len() - config.input_dirs.len());
		} else if backgrounds.len() < config.input_dirs.len() {
			// Weird, more than one background folder but fewer than one for each collection
			warnln!("Not enough background directories given. Reusing them in order");
		}

		for i in 0..config.input_dirs.len() {
			let dir = &config.input_dirs[i];
			let picture_paths = crate::mi::fs::list_dir(dir);
			let background_pictures = backgrounds[i % backgrounds.len()].clone();
			let title = config.collection_titles[i].clone();

			collections.push(
				Collection::new(
					title, picture_paths, background_pictures, config.tumb_size, config.display_size, config.background_size, config.image_name_titles
				)
			);
		}


		for i in 0..collections.len() {
			collections[i].pictures = check_and_filter(&collections[i].pictures);
			collections[i].backgrounds = check_and_filter(&collections[i].backgrounds);
		}


		Ok(Gallery{
			extension: String::from(PICTURE_EXTENSION),
			archives: HashMap::new(),
			collections: collections,
		})
	}


	pub fn create_images(&self, output_dir: &PathBuf, quality: u8, method: &String) {

		let pool = ThreadPool::new(8);

		for i in 0..self.collections.len() {
			self.collections[i].create_images(&pool, output_dir, quality, method)
		}

		info!("Working on pictures... {} left.\r", pool.queued_count() + pool.active_count());
		let mut i = 0;
		while pool.queued_count() + pool.active_count() > 0 {
			i = i + 1;
			let n = i % 4;
			info!("Working on pictures{}{} {} left. \r", ".".repeat(n), " ".repeat(3 - n), pool.queued_count() + pool.active_count());
			std::thread::sleep(std::time::Duration::from_millis(500));
		}
		infoln!("Working on pictures. Done         ");
		pool.join();
	}

	pub fn create_archive_full(&mut self, output_dir: &PathBuf) {
		let archive_path = std::path::PathBuf::from(output_dir).join(FULL_ARCHIVE_PATH);
		self.archives.insert(String::from("_full_"), String::from(FULL_ARCHIVE_PATH));

		let archive_file = std::fs::File::create(archive_path).unwrap();
		let mut zip = zip::ZipWriter::new(archive_file);
		let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		for c in &self.collections {
			let dir_name = crate::mi::fs::sanitize(c.title.as_str());
			zip.add_directory(&dir_name, options.clone()).unwrap();
			// println!("Starting dir {}", dir_name);
			for p in &c.pictures {
				let path = p.source_path.clone();
				let pic = path.file_name().unwrap().to_str().unwrap();
				// println!("Starting file {}", pic);
				let file_path = format!("{}/{}", &dir_name, pic);
				zip.start_file(file_path, options).unwrap();
				let mut buf = std::fs::read(p.source_path.clone()).unwrap();
				zip.write_all(&mut buf).unwrap();
			}
		}
		zip.finish().unwrap();

	}

	pub fn include_json_data(&self, output_dir: &PathBuf) {
		let json = self.json();
		let json_path = std::path::PathBuf::from(output_dir).join("data.json");

		// Insert JSON data in the index.html
		let index_path = std::path::PathBuf::from(output_dir).join("index.html");
		let html = match std::fs::read(&index_path) {
			Ok(d) => d,
			Err(e) => {
				panic!("Could not read from {}: {}", index_path.to_str().unwrap(), e);
			}
		};

		let replaced_html = html.replace(b"/*{{BEGIN:data*/{}/*END:data}}*/", json.as_bytes());

		match std::fs::write(index_path, replaced_html) {
			Ok(_) => {},
			Err(e) => {
				panic!("Could not write to {}: {}", json_path.to_str().unwrap(), e);
			}
		}
	}

	pub fn json(&self) -> String {
		let mut output = Json::new(JsonType::Object);
		output.start_property("extension", JsonType::String)
			.value(&self.extension)
			.end();

		output.start_property("archives", JsonType::Object);
		for (k, p) in &self.archives {
			output.start_property(k, JsonType::String).value(p).end();
		}
		output.end(); // "archives"

		output.start_property("collections", JsonType::Array);

		for col in &self.collections {
			let dir_name = crate::mi::fs::clean_basename(&col.title);

			output.object();
			output.start_property("title", JsonType::String)
				.value(&col.title)
				.end();

			output.start_property("pictures", JsonType::Array);
			let pic_dir = std::path::PathBuf::new().join("c").join(&dir_name);
			for pic in &col.pictures {
				let path = pic_dir.join(&pic.target_basename);

				output.object();
				output.start_property("title", JsonType::String).value(&pic.title).end();
				output.start_property("path", JsonType::String).value(path.to_str().unwrap()).end();
				output.end(); // object
			}
			output.end(); // "pictures"

			output.start_property("backgrounds", JsonType::Array);
			let bg_dir = std::path::PathBuf::new().join("b").join(&dir_name);
			for bg in &col.backgrounds {
				let path = bg_dir.join(&bg.target_basename);

				output.object();
				output.start_property("title", JsonType::String).value(&bg.title).end();
				output.start_property("path", JsonType::String).value(path.to_str().unwrap()).end();
				output.end(); // object
			}
			output.end(); // "backgrounds"

			output.end(); // object
		}

		output.end(); // "collections"


		output.stringify()
	}
}


#[derive(Debug, Clone)]
pub struct Picture {
	title: String,
	source_path: PathBuf,
	target_basename: String,
}



#[derive(Debug)]
pub struct Collection {
	pub title: String,
	pub pictures: Vec<Picture>,
	pub backgrounds: Vec<Picture>,

	pub res_thumb: Resolution,
	pub res_display: Resolution,
	pub res_background: Resolution,
}


impl Collection {
	pub fn new(title: String, picture_paths: Vec<String>, background_paths: Vec<String>, res_thumb: Resolution, res_display: Resolution, res_background: Resolution, use_filenames_as_titles: bool) -> Collection{
		let mut pictures = Vec::new();
		for p in &picture_paths {
			let source_path = PathBuf::from(p);
			let target_basename = clean_basename(p);
			let title = String::from(if use_filenames_as_titles { source_path.file_stem().unwrap().to_str().unwrap() } else { "" });

			pictures.push(Picture{ title, source_path, target_basename });
		}

		let mut backgrounds = Vec::new();
		for b in &background_paths {
			let source_path = PathBuf::from(b);
			let target_basename = clean_basename(b);
			let title = String::from("");

			backgrounds.push(Picture{ title, source_path, target_basename });
		}

		Collection{ title, pictures, backgrounds, res_thumb, res_display, res_background }
	}

	pub fn create_images(&self, pool: &ThreadPool, output_dir: &PathBuf, quality: u8, method: &String) {
		let dir_name = crate::mi::fs::clean_basename(&self.title);
		// Create collection directory
		let mut col_dir = PathBuf::from(output_dir);
		col_dir.push("c");
		col_dir.push(&dir_name);
		match std::fs::create_dir_all(&col_dir) {
			Ok(_) => {},
			Err(s) => { panic!("{}", s); }
		}

		// Create background directory
		let mut bg_dir = PathBuf::from(output_dir);
		bg_dir.push("b");
		bg_dir.push(&dir_name);
		match std::fs::create_dir_all(&bg_dir) {
			Ok(_) => {},
			Err(s) => { panic!("{}", s); }
		}

		self.resize_and_copy(&pool, &col_dir, &self.pictures, quality, method);
		self.resize_and_copy_bg(&pool, &bg_dir, &self.backgrounds, quality, method);
	}

	fn resize_and_copy(&self, pool: &ThreadPool, target_dir: &PathBuf, pictures: &Vec<Picture>, quality: u8, method: &String) {
		for p in pictures {
			let source = p.source_path.clone();

			let mut target = target_dir.clone();
			target.push(p.target_basename.clone());

			let ext = source.extension().unwrap().to_str().unwrap();

			let thumb = target.with_extension(format!("thumb.{}", ext));
			let display = target.with_extension(format!("disp.{}", ext));

			let source_thumb = source.clone();
			let target_thumb = thumb.clone();
			let res_thumb = self.res_thumb.clone();
			let method_thumb = String::from(method);
			pool.execute(move || {
				debugln!("Resize {} \t=> {} ({:?})", source_thumb.to_str().unwrap(), target_thumb.to_str().unwrap(), res_thumb);
				crate::mi::img::resize(&source_thumb, &target_thumb, res_thumb, quality, &method_thumb);
			});

			let source_display = source.clone();
			let target_display = display.clone();
			let res_display = self.res_display.clone();
			let method_display = String::from(method);
			pool.execute(move || {
				debugln!("Resize {} \t=> {} ({:?})", source_display.to_str().unwrap(), target_display.to_str().unwrap(), res_display);
				crate::mi::img::resize(&source_display, &target_display, res_display, quality, &method_display);
			});

			let source_full = source;
			let mut target_full = target;
			target_full.set_extension(PICTURE_EXTENSION);
			pool.execute(move || {
				debugln!("Recode {} \t=> {}", source_full.to_str().unwrap(), target_full.to_str().unwrap());
				crate::mi::img::recode(&source_full, &target_full, quality);
			});
		}
	}

	fn resize_and_copy_bg(&self, pool: &ThreadPool, target_dir: &PathBuf, pictures: &Vec<Picture>, quality: u8, method: &String) {
		for p in pictures {
			let source = p.source_path.clone();
			let res = self.res_background.clone();
			let method = String::from(method);
			let mut target = target_dir.clone();
			target.push(p.target_basename.clone());
			target.set_extension(PICTURE_EXTENSION);

			pool.execute(move || {
				debugln!("Resize {} \t=> {} ({:?})", source.to_str().unwrap(), target.to_str().unwrap(), res);
				crate::mi::img::resize(&source, &target, res, quality, &method);
			});
		}
	}



}
