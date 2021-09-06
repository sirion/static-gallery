
use std::path::PathBuf;
use structopt::StructOpt;
use crate::{infoln, warnln};
use crate::gallery::Gallery;
use crate::mi::img::Resolution;

/// Generate a static picture gallery using the given template
#[derive(StructOpt, Debug)]
#[structopt(name = "miGallery")]
/// Generates a static gallery from the given inputs.
///
/// For each collection the options --input --background and --title should be set.
///
/// The number of directories for --input and --background and the --title options should match.
/// If only one background option is given, the pictures in that folder will be used for all collections.
///
/// Examples:
///
///  - mi_gallery -i dir1 -b dir2 -t \"Collection 01\" -i dir3 -b dir4 -t \"Collection 02\" -o outdir
///  - mi_gallery -i dir1 -b dir2 -t \"Pictures\" -o outdir
///
pub struct Configuration {
	/// The input directories for the picture collections
	#[structopt(short = "i", long = "input")]
	pub input_dirs: Vec<PathBuf>,

	/// The input directories for the background images
	#[structopt(short = "b", long = "background")]
	pub background_dirs: Vec<PathBuf>,

	/// The input directories for the background images
	#[structopt(short = "t", long = "title")]
	pub collection_titles: Vec<String>,

	/// The output directory for the generated gallery
	#[structopt(short = "o", long = "output")]
	pub output_dir: PathBuf,

	/// The directory of the template to be used for the gallery
	#[structopt(short = "p", long = "template")]
	pub template_dir: PathBuf,

	/// Whether to clear the output irectory
	#[structopt(short = "c", long = "clean")]
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

	// /// Disables any output (including errors)
	// #[structopt(short = "s", long = "silent")]
    // silent: bool,


	#[structopt(skip)]
	delete_output_dir: bool,
	#[structopt(skip)]
	create_output_dir: bool,


	#[structopt(skip)]
	pub gallery: Gallery,
}



impl Configuration {
	pub fn from_cli() -> Result<Configuration, String> {
		let mut config = Configuration::from_args();

		// Verbose is 0 by default, which as log level would be silent, but we wand errors to be shown by default
		config.verbose = config.verbose + 1;
		crate::mi::logger::set_level(config.verbose);
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
					infoln!("Output directory removed");
					true
				},
				Err(e) => {
					errors.push(format!("Could not remove output directory: {}", e));
					false
				}
			};

			if removed {
				match std::fs::create_dir_all(std::path::Path::new(&self.output_dir)) {
					Ok(_) => {
						infoln!("Output directory created");
					},
					Err(e) => {
						errors.push(format!("Output directory could not be created: {}", e));
					}
				};
			}
		}





		if errors.len() > 0 {
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
			infoln!("Using {} threads for image resizing", self.threads);
		}


		if self.jpeg_quality < 1 || self.jpeg_quality > 100 {
			errors.push(String::from("Jpeg quality must be between 1 and 100"));
		}

		if self.input_dirs.len() == 0 {
			errors.push(String::from("No input directories specified"));
		}


		match self.resize_method.as_str() {
			"lanczos3" => {},
			"gaussian" => {},
			"nearest" => {},
			"cubic" => {},
			"linear" => {},
			_ => {
				errors.push(format!("Invalid resize method \"{}\". Valid options: \"lanczos3\", \"gaussian\" and \"nearest\"", self.resize_method));
			},
		};



		// Validate background folders exists
		for dir in &self.background_dirs {
			let pictures = crate::mi::fs::list_dir(dir);
			if pictures.len() == 0 {
				errors.push(format!("Background directory {} is empty or does not exist", dir.to_str().unwrap()));
				continue;
			}
		}


		// Validate collection titles were given
		if self.collection_titles.len() > self.input_dirs.len() {
			// Weird, more than one collection title for each collection
			warnln!("Too many collection titles given. Ignoring the last {}", self.collection_titles.len() - self.input_dirs.len());
		} else if self.collection_titles.len() < self.input_dirs.len() {
			// Weird, more than one background folder but fewer than one for each collection
			warnln!("Not enough collection titles given. Creating: ");
			for i in (self.collection_titles.len() + 1)..=self.input_dirs.len() {
				let title = format!("Collection {}", i);
				warnln!(" - \"{}\" ", title);
				&self.collection_titles.push(String::from(title));
			}
		}



		// Validate input folders exists
		for i in 0..self.input_dirs.len() {
			let dir = &self.input_dirs[i];
			let pictures = crate::mi::fs::list_dir(dir);

			if pictures.len() == 0 {
				errors.push(format!("Input directory {} is empty or does not exist", dir.to_str().unwrap()));
				continue;
			}
		}

		// Validate template folder exists
		if !crate::mi::fs::dir_exists(&self.template_dir) {
			errors.push(format!("Template directory is not a directory: {}", self.template_dir.to_str().unwrap()));
		}
		// Validate template is valid
		if !crate::mi::fs::file_exists(&self.template_dir.join("index.html")) {
			errors.push(format!("Template directory is not a directory: {}", self.template_dir.to_str().unwrap()));
		}




		if !self.clean_output {
			// Validate output folder does not exists and can be created or is empty.
			if crate::mi::fs::list_dir(&self.output_dir).len() > 0 {
				errors.push(format!("Output directory is not empty"));
			}

			if !crate::mi::fs::dir_exists(&self.output_dir) {
				self.create_output_dir = true;
			}
		} else {
			self.delete_output_dir = true;
			self.create_output_dir = true;
		}




		if errors.len() > 0 {
			Err(errors.join("\n"))
		} else {
			Ok(0)
		}
	}
}
