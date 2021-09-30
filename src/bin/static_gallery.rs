mod configuration;
mod test;

use configuration::Configuration;
use gallery::Gallery;
use mi::logger::errorln;

// TODO: Image ordering by Exif date, file timestamp or original name
// TODO: Document APIs inline
// TODO: Optimize output if requested
// TODO: Allow change of gallery browser title
// IDEA: Define additional changeable theme parameters
// IDEA: Offer to include originals
// IDEA: Create collection archives
// IDEA: Validate input files are valid? (Warn if non-images are found)
// IDEA: List current gallery status

fn main() {
	// Fill CLI options
	let config: Configuration = Configuration::from_cli().unwrap();

	let mut gallery = match config.update {
		true => Gallery::from(&config.output_dir),
		false => Gallery::new(),
	};

	gallery
		.fill(config.collections, config.image_name_titles)
		.unwrap();

	// Create output images (resized versions)
	gallery
		.create_images(
			&config.output_dir,
			config.jpeg_quality,
			&config.resize_method,
			config.threads,
		)
		.unwrap();

	if !config.update {
		// Copy template
		match mi::fs::copy_recursively(&config.template_dir, &config.output_dir) {
			Ok(_) => (),
			Err(e) => {
				errorln(format!("Could not copy template directory: {}", e));
			}
		};
	}

	// Create archive if requested
	if config.create_full_archive {
		gallery.create_archive_full(&config.output_dir);
	}

	// Generate and include JSON structure
	gallery.include_json_data(&config.output_dir);
}
