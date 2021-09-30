use mi::logger::infoln;

use gallery::CollectionInput;
use mi::img::Resolution;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

/// Generate a static picture gallery using the given template
#[derive(StructOpt, Debug)]
#[structopt(name = "static_gallery")]
/// Generates a static gallery from the given inputs.
///
/// For each collection the options --input --background and --title should be set.
///
/// The number of directories for --input and --background and the --title options should match.
/// If no backgrounds should be used for a collection, use the value "-" indicating no background directory.
///
/// Examples:
///
/// Create new gallery:
///
///  - Create a new gallery with one collection
///    static_gallery -o out/ -c "in1/;bg1/;Pictures"
///
///  - Create a new gallery with two collections (the second collection without backgrounds)
///    static_gallery -o out/ -c "in1/;bg1/;Collection 01" -c "in2/;-;Collection 02"
///
/// Update existing gallery:
///  - Add a new collection
///    static_gallery -u -o out/ -c "in1/;bg1/;New Collection"
///
///  - Add pictures to existing collection (collection with same title must already exist, else it will be created as new)
///    static_gallery -u -o out/ -c "in2/;-;Collection 01"
///
///  - Add backgrounds to existing collection (collection with same title must already exist, else it will be created as new)
///    static_gallery -u -o out/ -c "-;bg2/;Collection 01"
///
pub struct Configuration {
	/// Collection input as "[input directory];[background directory],[collection title]". Examples: "in/;bg/;Col 1", "in/;-;Col 2"
	#[structopt(short = "c", long = "collection")]
	pub collections: Vec<CollectionInput>,

	/// The output directory for the generated gallery
	#[structopt(short = "o", long = "output")]
	pub output_dir: PathBuf,

	/// The directory of the template to be used for the gallery
	#[structopt(short = "p", long = "template")]
	pub template_dir: PathBuf,

	/// Whether to clear the output irectory
	#[structopt(short = "r", long = "remove-output")]
	pub clean_output: bool,

	/// Whether to create an archive (downloadable zip-file) with the original pictures
	#[structopt(short = "a", long = "archive")]
	pub create_full_archive: bool,

	/// The size of the small picture versions (thumbnails)
	#[structopt(long = "thumb-size", default_value = "960x540")]
	pub tumb_size: Resolution,

	/// The size of the display picture versions
	#[structopt(long = "display-size", default_value = "2560x1440")]
	pub display_size: Resolution,

	/// The size of the backgroun picture versions
	#[structopt(long = "background-size", default_value = "2560x1440")]
	pub background_size: Resolution,

	/// Image resize method. Valid methods: "lanczos3", "gaussian", "nearest", "cubic", "linear"
	#[structopt(long = "resize-method", default_value = "lanczos3")]
	pub resize_method: String,

	/// Quality of the output images 1-100
	#[structopt(long = "jpeg-quality", default_value = "75")]
	pub jpeg_quality: u8,

	/// Number of concurrent threads to use for image resizing.
	/// If set to 0 it uses the number of available logical cores.
	#[structopt(long = "threads", default_value = "0")]
	pub threads: usize,

	/// Increases the log level. By default only errors are shown.
	/// Levels: Error, Warning, Info, Debug
	#[structopt(short = "v", long = "verbose", parse(from_occurrences))]
	pub verbose: u8,

	/// When set to true the image names (without extensions) are used as picture titles
	#[structopt(long = "image-name-titles")]
	pub image_name_titles: bool,

	/// Update gallery (add new pictures to existing gallery) overwrites pictures with the same file name in the same collection
	#[structopt(short = "u", long = "update")]
	pub update: bool,

	// /// Disables any output (including errors)
	// #[structopt(short = "s", long = "silent")]
	// silent: bool,
	#[structopt(skip)]
	delete_output_dir: bool,
	#[structopt(skip)]
	create_output_dir: bool,
}

impl Configuration {
	pub fn from_cli() -> Result<Configuration, String> {
		let mut config = Configuration::from_args();

		// Verbose is 0 by default, which as log level would be silent, but we wand errors to be shown by default
		config.verbose += 1;
		mi::logger::set_level(config.verbose);
		// println!("Set log_level to {}", config.verbose);

		config.validate()?;
		config.init()?;

		Ok(config)
	}

	fn init(&mut self) -> Result<u8, String> {
		// Run initialization tasks if any

		let mut errors: Vec<String> = Vec::new();

		// Remove output files recursively
		if self.clean_output {
			let removed = match std::fs::remove_dir_all(&self.output_dir) {
				Ok(_) => {
					infoln("Output directory removed".to_string());
					true
				}
				Err(e) => {
					errors.push(format!("Could not remove output directory: {}", e));
					false
				}
			};

			if removed {
				match std::fs::create_dir_all(std::path::Path::new(&self.output_dir)) {
					Ok(_) => {
						infoln("Output directory created".to_string());
					}
					Err(e) => {
						errors.push(format!("Output directory could not be created: {}", e));
					}
				};
			}
		}

		if errors.is_empty() {
			Err(errors.join("\n"))
		} else {
			Ok(0)
		}
	}

	fn validate(&mut self) -> Result<u8, String> {
		let mut errors: Vec<String> = Vec::new();

		if self.threads == 0 {
			self.threads = num_cpus::get();
			// self.threads = num_cpus::get_physical();
			infoln(format!("Using {} threads for image resizing", self.threads));
		}

		if self.jpeg_quality < 1 || self.jpeg_quality > 100 {
			errors.push(String::from("Jpeg quality must be between 1 and 100"));
		}

		if self.collections.is_empty() {
			errors.push(String::from("No collections specified"));
		}

		// Validate collections
		for col in &self.collections {
			if col.title.is_empty() || col.title == "-" {
				errors.push("Collections must have valid titles".to_string());
			}

			if !self.update && col.input_dir.is_none() {
				errors.push(format!(
					"New collection \"{}\" does not have an input directory",
					col.title
				));
			}

			if col.input_dir.is_none() && col.background_dir.is_none() {
				errors.push(format!(
					"New collection \"{}\" has neither an input nor background directory",
					col.title
				));
			}

			if col.input_dir.is_some() && !gallery::contains_images(col.input_dir.as_ref().unwrap())
			{
				errors.push(format!(
					"Input directory for collection \"{}\" does not contain images: {}",
					col.title,
					col.input_dir.as_ref().unwrap().to_string_lossy()
				));
			}

			if col.background_dir.is_some()
				&& !gallery::contains_images(col.background_dir.as_ref().unwrap())
			{
				errors.push(format!(
					"Background directory for collection \"{}\" does not contain images: {}",
					col.title,
					col.background_dir.as_ref().unwrap().to_string_lossy()
				));
			}
		}

		match self.resize_method.as_str() {
			"lanczos3" => {}
			"gaussian" => {}
			"nearest" => {}
			"cubic" => {}
			"linear" => {}
			_ => {
				errors.push(format!("Invalid resize method \"{}\". Valid options: \"lanczos3\", \"gaussian\" and \"nearest\"", self.resize_method));
			}
		};

		// Validate template folder exists
		if !Path::new(&self.template_dir).is_dir() {
			errors.push(format!(
				"Template directory is not a directory: {}",
				self.template_dir.to_str().unwrap()
			));
		}
		// Validate template is valid
		if !self.template_dir.join("index.html").is_file() {
			errors.push(format!(
				"Template directory is not a directory: {}",
				self.template_dir.to_str().unwrap()
			));
		}

		if self.update && self.clean_output {
			errors.push(String::from(
				"Options --clean und --update are mutually exclusive. Choose only one of them.",
			));
		}

		if self.update && !self.output_dir.join("index.html").is_file() {
			errors.push(format!(
				"No index.html found in the output folder ({}), cannot update.",
				self.output_dir.to_str().unwrap()
			));
		}

		let output_dir_exists = Path::new(&self.output_dir).is_dir();
		let output_dir_empty = if output_dir_exists {
			mi::fs::list_dir(&self.output_dir).is_empty()
		} else {
			true
		};

		if self.update {
			// Update existing gallery.
			self.delete_output_dir = false;
			self.create_output_dir = !output_dir_exists;
		} else if self.clean_output {
			// Delete gallery and start fresh
			self.delete_output_dir = output_dir_exists;
			self.create_output_dir = true;
		} else if !output_dir_exists || output_dir_empty {
			// Neither update nor delete output directory. Must not exist or be empty
			self.create_output_dir = !output_dir_exists;
		} else {
			errors.push(format!(
				"Output directory already exists: {}",
				&self.output_dir.to_str().unwrap()
			));
		}

		if errors.is_empty() {
			Err(errors.join("\n"))
		} else {
			Ok(0)
		}
	}
}
