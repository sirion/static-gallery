use mi::logger::debugln;
use crate::gallery::GalleryImages;
use crate::gallery::Image;
use crate::gallery::Picture;
// use crate::mi::fs::clean_basename;
use crate::mi::img::Resolution;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct CollectionInput {
	pub title: String,
	pub name: String,
	pub input_dir: Option<PathBuf>,
	pub background_dir: Option<PathBuf>,
	pub exists: bool,
}

impl std::str::FromStr for CollectionInput {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<&str> = s.splitn(3, ";").collect();

		if parts.len() != 3 {
			return Err(String::from(
				"Invalid collection argument, must have three parts (input;backgrounds;title)",
			));
		}

		let input_dir = match parts[0] {
			"-" => None,
			_ => {
				let path = PathBuf::from_str(parts[0]).unwrap();
				if !path.is_dir() {
					return Err(format!("Input directory {} does not exist", parts[0]));
				}

				Some(path)
			}
		};
		let background_dir = match parts[1] {
			"-" => None,
			_ => {
				let path = PathBuf::from_str(parts[1]).unwrap();
				if !path.is_dir() {
					return Err(format!("Background directory {} does not exist", parts[1]));
				}

				Some(path)
			}
		};
		let title = String::from(parts[2]);
		let name = crate::mi::fs::sanitize(title.as_str());

		Ok(CollectionInput {
			title,
			name,
			input_dir,
			background_dir,
			exists: false,
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
	/// Collection title as shown to the user
	pub title: String,

	/// Collection basename (directory name) derived from title
	pub name: String,

	pub pictures: Vec<Picture>,
	pub backgrounds: Vec<Image>,
}

impl Collection {
	pub fn new(
		name: String,
		title: String,
		picture_paths: Vec<PathBuf>,
		background_paths: Vec<PathBuf>,
		use_filenames_as_titles: bool,
	) -> Collection {
		let mut pictures: Vec<Picture> = GalleryImages::from(&picture_paths);
		let backgrounds = GalleryImages::from(&background_paths);

		if !use_filenames_as_titles {
			pictures.clear_titles();
		}

		Collection {
			title,
			name,
			pictures,
			backgrounds,
		}
	}

	pub fn append(&mut self, mut other: Collection) {
		self.pictures.append(&mut other.pictures);
		self.backgrounds.append(&mut other.backgrounds);
	}

	pub fn create_images(
		&mut self,
		pool: &ThreadPool,
		output_dir: &PathBuf,
		quality: u8,
		method: &String,
		res_thumb: Resolution,
		res_display: Resolution,
		res_background: Resolution,
	) -> Result<(), Box<dyn std::error::Error>> {
		// Create picture directory if not already existing
		let pictures_dir = output_dir.join(crate::gallery::PICTURES_DIR_NAME);
		std::fs::create_dir_all(&pictures_dir)?;

		// TODO: Do not overwrite pictures with the same name but different content
		for p in self.pictures.iter_mut() {
			if !p.image.update {
				continue;
			}

			let source = p.image.source_path.clone();

			let mut target = pictures_dir.clone();
			let file_stem = p.image.basename.clone();
			target.push(&file_stem);

			let ext = crate::gallery::PICTURE_EXTENSION;

			let thumb = target.with_extension(format!("thumb.{}", ext));
			let display = target.with_extension(format!("disp.{}", ext));

			let source_thumb = source.clone();
			let res_thumb = res_thumb.clone();
			let method_thumb = String::from(method);

			let source_display = source.clone();
			let res_display = res_display.clone();
			let method_display = String::from(method);

			let source_full = source.clone();

			let mut target_thumb = thumb.clone();
			let mut target_display = display.clone();
			let mut target_full = target.with_extension(ext);

			let mut i = 0;
			while target_thumb.exists() || target_display.exists() || target_full.exists() {
				i += 1;
				target_thumb.set_file_name(format!("{}-{}.thumb.{}", &file_stem, i, ext));
				target_display.set_file_name(format!("{}-{}.disp.{}", &file_stem, i, ext));
				target_full.set_file_name(format!("{}-{}.{}", &file_stem, i, ext));
			}
			p.image.basename = String::from(target_full.file_stem().unwrap().to_string_lossy());

			pool.execute(move || {
				debugln(format!(
					"Resize {} \t=> {} ({:?})",
					source_thumb.to_str().unwrap(),
					target_thumb.to_str().unwrap(),
					res_thumb
				));
				crate::mi::img::resize(
					&source_thumb,
					&target_thumb,
					res_thumb,
					quality,
					&method_thumb,
				);
			});

			pool.execute(move || {
				debugln(format!(
					"Resize {} \t=> {} ({:?})",
					source_display.to_str().unwrap(),
					target_display.to_str().unwrap(),
					res_display
				));
				crate::mi::img::resize(
					&source_display,
					&target_display,
					res_display,
					quality,
					&method_display,
				);
			});

			pool.execute(move || {
				debugln(format!(
					"Recode {} \t=> {}",
					source_full.to_str().unwrap(),
					target_full.to_str().unwrap()
				));
				crate::mi::img::recode(&source_full, &target_full, quality);
			});
		}

		// TODO: Do not overwrite pictures with the same name but different content
		for p in self.backgrounds.iter_mut() {
			if !p.update {
				continue;
			}

			let source = p.source_path.clone();
			let method = String::from(method);
			let ext = crate::gallery::PICTURE_EXTENSION;
			let mut target = pictures_dir.clone();
			let file_stem = p.basename.clone();
			target.push(&file_stem);
			let mut target_full = target.with_extension(ext);
			target.set_extension(format!("bg.{}", ext));

			let mut i = 0;
			while target.exists() {
				i += 1;
				target.set_file_name(format!("{}-{}.bg.{}", &file_stem, i, ext));
				target_full.set_file_name(format!("{}-{}.{}", &file_stem, i, ext));
			}
			p.basename = String::from(target_full.file_stem().unwrap().to_string_lossy());

			pool.execute(move || {
				debugln(format!(
					"Resize {} \t=> {} ({:?})",
					source.to_str().unwrap(),
					target.to_str().unwrap(),
					res_background
				));
				crate::mi::img::resize(&source, &target, res_background, quality, &method);
			});
		}

		Ok(())
	}
}
