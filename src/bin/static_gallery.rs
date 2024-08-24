use mi;

mod configuration;
mod gallery;

mod test;

use configuration::Configuration;
use gallery::Gallery;

// TODO: Video Support

// TODO (hauer template): Make sure backgrounds don't run out
// TODO: Optimize Rotation detection (currently reading images twice)

// TODO: Add Unit Tests
// TODO: Document APIs inline
// TODO: Add to existing without recreating images
// TODO: Detect dupicate backgrounds in collections
// TODO: Optimize output if requested
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

	gallery.remove_duplicates();

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
		crate::mi::fs::copy_recursively(&config.template_dir, &config.output_dir);
	}

	// Create archive if requested
	if config.create_full_archive {
		gallery.create_archive_full(&config.output_dir);
	}

	// Generate and include JSON structure
	gallery.include_json_data(&config.output_dir);
}
