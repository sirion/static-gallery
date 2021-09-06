mod mi;
mod configuration;
mod gallery;

use configuration::Configuration;
use mi::proc::exit_on_error;
use gallery::Gallery;

// TODO: Add to existing without recreating images
// TODO: Use file names as image titles option
// TODO: Detect dupicate backgrounds in collections
// TODO: Optimize output if requested
// IDEA: Create collection archives
// IDEA: Validate input files are valid? (Warn if non-images are found)


fn main() {
	// Fill CLI options
	let config: Configuration = exit_on_error(Configuration::from_cli());

	let mut gallery = exit_on_error(Gallery::from(&config));

	// Create output images (resized versions)
	gallery.create_images(&config.output_dir, config.jpeg_quality, &config.resize_method);

	// Copy template
	crate::mi::fs::copy_recursively(&config.template_dir, &config.output_dir);

	// Create archive if requested
	if config.create_full_archive {
		gallery.create_archive_full(&config.output_dir);
	}

	// Generate and include JSON structure
	gallery.include_json_data(&config.output_dir);
}

