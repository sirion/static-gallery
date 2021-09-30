use super::GalleryImages;
use super::Image;
use super::Picture;
use mi::img::Resolution;
use mi::{log_debug, log_error};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
		let parts: Vec<&str> = s.splitn(3, ';').collect();

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
		let name = mi::fs::sanitize(title.as_str());

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

	pub fn remove_resource(&mut self, name: &str) {
		let mut backgrounds: Vec<Image> = Vec::with_capacity(self.backgrounds.len());
		std::mem::swap(&mut self.backgrounds, &mut backgrounds);
		for p in backgrounds {
			if p.hash.to_string() != name {
				self.backgrounds.push(p);
			}
		}

		let mut pictures: Vec<Picture> = Vec::with_capacity(self.pictures.len());
		std::mem::swap(&mut self.pictures, &mut pictures);
		for p in pictures {
			if p.image.hash.to_string() != name {
				self.pictures.push(p);
			}
		}
	}

	#[allow(clippy::too_many_arguments)] // TODO: Remove atttribute and rewrite
	pub fn create_images(
		&mut self,
		failed: Arc<Mutex<Vec<(String, String)>>>,
		pool: &ThreadPool,
		output_dir: &Path,
		quality: u8,
		method: &str,
		res_thumb: Resolution,
		res_display: Resolution,
		res_background: Resolution,
	) -> Result<(), Box<dyn std::error::Error>> {
		// Create picture directory if not already existing
		let pictures_dir = output_dir.join(super::PICTURES_DIR_NAME);
		std::fs::create_dir_all(&pictures_dir)?;

		for p in self.pictures.iter_mut() {
			if !p.image.update {
				continue;
			}

			let source = p.image.source_path.clone();

			let mut target = pictures_dir.clone();
			let file_stem = format!("{}", p.image.hash);
			target.push(&file_stem);

			let ext = super::PICTURE_EXTENSION;

			let thumb = target.with_extension(format!("thumb.{}", ext));
			let display = target.with_extension(format!("disp.{}", ext));

			let source_thumb = source.clone();
			let method_thumb = String::from(method);

			let source_display = source.clone();
			let method_display = String::from(method);

			let source_full = source.clone();

			let target_thumb = thumb.clone();
			let target_display = display.clone();
			let target_full = target.with_extension(ext);

			if !target_thumb.exists() {
				let f = failed.clone();
				let n = (self.name.clone(), file_stem.clone());
				let s: String = p.image.source_path.clone().to_string_lossy().into();
				pool.execute(move || {
					log_debug!(
						"Resize {} \t=> {} ({:?})",
						source_thumb.to_str().unwrap(),
						target_thumb.to_str().unwrap(),
						res_thumb
					);

					match mi::img::resize_jpg(
						&source_thumb,
						&target_thumb,
						res_thumb,
						quality,
						&method_thumb,
					) {
						Ok(_) => {}
						Err(e) => {
							// Remove picture from gallery
							log_error!("Could not process picture {}: {}", s, e);
							let mut guard = f.lock().unwrap();
							guard.push(n);
						}
					};
				});
			}

			if !target_display.exists() {
				let f = failed.clone();
				let n = (self.name.clone(), file_stem.clone());
				let s: String = p.image.source_path.to_string_lossy().into();
				pool.execute(move || {
					log_debug!(
						"Resize {} \t=> {} ({:?})",
						source_display.to_str().unwrap(),
						target_display.to_str().unwrap(),
						res_display
					);
					match mi::img::resize_jpg(
						&source_display,
						&target_display,
						res_display,
						quality,
						&method_display,
					) {
						Ok(_) => {}
						Err(e) => {
							// Remove picture from gallery
							log_error!("Could not process picture {}: {}", s, e);
							let mut guard = f.lock().unwrap();
							guard.push(n);
						}
					};
				});
			}

			if !target_full.exists() {
				let f = failed.clone();
				let n = (self.name.clone(), file_stem.clone());
				let s: String = p.image.source_path.to_string_lossy().into();
				pool.execute(move || {
					log_debug!(
						"Recode {} \t=> {}",
						source_full.to_str().unwrap(),
						target_full.to_str().unwrap()
					);
					match mi::img::recode_jpg(&source_full, &target_full, quality) {
						Ok(_) => {}
						Err(e) => {
							// Remove picture from gallery
							log_error!("Could not process picture {}: {}", s, e);
							let mut guard = f.lock().unwrap();
							guard.push(n);
						}
					};
				});
			}
		}

		for p in self.backgrounds.iter_mut() {
			if !p.update {
				continue;
			}

			let source = p.source_path.clone();
			let method = String::from(method);
			let ext = super::PICTURE_EXTENSION;
			let mut target = pictures_dir.clone();
			let file_stem = format!("{}", p.hash);

			target.push(&file_stem);
			target.set_extension(format!("bg.{}", ext));

			if !target.exists() {
				let f = failed.clone();
				let n = (self.name.clone(), file_stem.clone());
				let s: String = p.source_path.clone().to_string_lossy().into();
				pool.execute(move || {
					log_debug!(
						"Resize {} \t=> {} ({:?})",
						source.to_str().unwrap(),
						target.to_str().unwrap(),
						res_background
					);
					match mi::img::resize_jpg(&source, &target, res_background, quality, &method) {
						Ok(_) => {}
						Err(e) => {
							// Remove picture from gallery
							log_error!("Could not process picture {}: {}", s, e);
							let mut guard = f.lock().unwrap();
							guard.push(n);
						}
					};
				});
			}
		}

		Ok(())
	}
}
