use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
	pub title: String,

	#[serde(flatten)]
	pub image: Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
	// Basename of the image
	#[serde(rename = "path")]
	pub basename: String,

	#[serde(skip)]
	pub update: bool,

	#[serde(skip)]
	pub source_path: PathBuf,

	pub original_hash: u64,
}


