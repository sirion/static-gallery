use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use image::GenericImageView;
// use crate::debug;

const RESOLUTION_MIN:Resolution = Resolution{ width: 150, height: 150 };
// const RESOLUTION_QFHD:Resolution = Resolution{ width: 960, height: 540 };
// const RESOLUTION_FHD:Resolution = Resolution{ width: 1920, height: 1080 };
// const RESOLUTION_QHD:Resolution = Resolution{ width: 2560, height: 1440 };
// const RESOLUTION_UHD:Resolution = Resolution{ width: 3840, height: 2160 };


pub fn resize(source: &PathBuf, target: &PathBuf, resolution: Resolution, quality: u8, method: &String) {
	let method = match method.as_str() {
		"lanczos3" => image::imageops::FilterType::Lanczos3,
		"gaussian" => image::imageops::FilterType::Gaussian,
		"nearest" => image::imageops::FilterType::Nearest,
		"cubic" => image::imageops::FilterType::CatmullRom,
		"linear"=> image::imageops::FilterType::Triangle,
		_ => panic!("Invalid resize method: {}", method),
	};

	let image = image::open(source).unwrap();

	let ratio = image.width() as f64 / image.height() as f64;
	let mut width = resolution.width as f64;
	let mut height = resolution.height as f64;

	if width / ratio > resolution.height as f64 {
		height = width / ratio;
	} else {
		width = height / ratio;
	}

	let new_image = image::imageops::resize(&image, width as u32, height as u32, method);

	let mut out = std::fs::File::create(target).unwrap();
	let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
	enc.encode_image(&new_image).unwrap();

}

pub fn recode(source: &PathBuf, target: &PathBuf, quality: u8) {
	let image = image::open(source).unwrap();

	let mut out = std::fs::File::create(target).unwrap();
	let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
	enc.encode_image(&image).unwrap();
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Resolution {
	pub width: u32,
	pub height: u32
}


impl Resolution {
	pub fn new(s: &str) -> Result<Resolution, String>{
		let split: Vec<&str> = s.split('x').collect();
		if split.len() == 2 {

			// eprintln!("Res Input: {}, {}", split[0], split[1]);

			let width: u32 = match split[0].trim().parse() {
				Ok(n) => n,
				Err(_) => {
					return Err(String::from("Invalid Resolution"));
				}
			};
			let height: u32 = match split[1].trim().parse() {
				Ok(n) => n,
				Err(_) => {
					return Err(String::from("Invalid Resolution"));
				}
			};

			if width < RESOLUTION_MIN.width || height < RESOLUTION_MIN.height {
				return Err(String::from("Resolution to low"))
			}

			Ok(Resolution{
				width,
				height
			})
		} else {
			Err(String::from("Invalid Resolution, must be in format WxH"))
		}
	}

	// pub fn to_string(&self) -> String {
	// 	format!("{}x{}", self.width, self.height)
	// }
}

impl std::str::FromStr for Resolution {
	type Err = String;

	fn from_str(s: &str) -> Result<Resolution, String> {
		Resolution::new(s)
	}

}



