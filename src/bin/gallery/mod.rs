mod gallery;
mod collection;
mod picture;

// use crate::mi::img::Resolution;

pub use gallery::Gallery;
pub use collection::Collection;
pub use collection::CollectionInput;
pub use picture::Picture;
pub use picture::Image;


pub const GALLERY_CONFIGURATION_VERSION: u16 = 1;
pub const FULL_ARCHIVE_PATH: &str = "Gallery.zip";

// TODO: Support more modern picture formats
pub const PICTURE_EXTENSION: &str = "jpg";

// pub const BACKGROUNDS_DIR_NAME: &str  = "b";
// pub const COLLECTIONS_DIR_NAME: &str  = "c";
pub const PICTURES_DIR_NAME: &str  = "p";

pub const PATTERM_DATA_START: &[u8] = b"/*{{BEGIN:data*/";
pub const PATTERM_DATA_END: &[u8] = b"/*END:data}}*/";

use crate::mi::img::Resolution;
pub const DEFAULT_RESOLUTION_THUMB: Resolution = Resolution{ width: 960, height: 540 };
pub const DEFAULT_RESOLUTION_DISPLAY: Resolution = Resolution{ width: 2560, height: 1440 };
pub const DEFAULT_RESOLUTION_BACKGROUND: Resolution = Resolution{ width: 2560, height: 1440 };


pub fn contains_images(dir: &std::path::PathBuf) -> bool {
	if !dir.is_dir() {
		return false;
	}

	for file in &crate::mi::fs::list_dir(&dir) {
		if valid_extension(file) {
			return true;
		}
	}

	false
}

pub trait GalleryImages {
	fn from(paths: &Vec<std::path::PathBuf>) -> Self;

	fn filter_valid(&mut self);

	fn clear_titles(&mut self);
}

impl GalleryImages for Vec<Picture> {
	fn from(paths: &Vec<std::path::PathBuf>) -> Vec<Picture> {
		let mut new = Vec::with_capacity(paths.len());

		for path in paths {
			let original_hash = crate::mi::fs::file_hash_quick(&path);
			// let basename = clean_basename(&source_path);
			let basename = format!("{}", original_hash);
			let source_path = path.to_path_buf();
			let title = String::from(source_path.file_stem().unwrap_or_default().to_string_lossy());

			new.push(Picture{
				title,
				image: Image{
					basename,
					source_path,
					original_hash,
					update: true,
				}
			});
		}

		new
	}

	fn filter_valid(&mut self) {
		let mut i = 0;
		let mut len = self.len();
		while i < len {
			if !valid_extension(&self[i].image.source_path) {
				self.remove(i);
				len = len - 1;
			} else {
				i = i + 1;
			}
		}
	}

	fn clear_titles(&mut self) {
		for pic in self {
			pic.title.clear();
		}
	}

}

impl GalleryImages for Vec<Image> {
	fn from(paths: &Vec<std::path::PathBuf>) -> Vec<Image> {
		let mut new = Vec::with_capacity(paths.len());

		for path in paths {
			let original_hash = crate::mi::fs::file_hash_quick(&path);
			// let basename = clean_basename(&source_path);
			let basename = format!("{}", original_hash);
			let source_path = path.to_path_buf();

			new.push(Image{
				basename,
				source_path,
				original_hash,
				update: true,
			});
		}

		new
	}

	fn filter_valid(&mut self) {

		let mut i = 0;
		let mut len = self.len();
		while i < len {
			if !valid_extension(&self[i].source_path) {
				self.remove(i);
				len = len - 1;
			} else {
				i = i + 1;
			}
		}
	}

	fn clear_titles(&mut self) {
		// Does not do anything for Vec<Image>
	}
}

fn valid_extension(path: &std::path::PathBuf) -> bool {
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
