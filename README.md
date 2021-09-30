# static-gallery

## Create static image galleries for the web

Use the `static_gallery` executable to generate a gallery that can be served on a webserver or a USB stick using only static resources.

## static_gallery command

	Generate a static picture gallery using the given template Generates a static gallery from the given inputs.

	For each collection the options --input --background and --title should be set.

	The number of directories for --input and --background and the --title options should match. If no backgrounds should be
	used for a collection, use the value "-" indicating no background directory.

	Examples:

	Create new gallery:

	- Create a new gallery with one collection
	  static_gallery -o out/ -c "in1/;bg1/;Pictures"

	- Create a new gallery with two collections (the second collection without backgrounds)
	  static_gallery -o out/ -c "in1/;bg1/;Collection 01" -c "in2/;-;Collection 02"

	Update existing gallery:

	- Add a new collection
	  static_gallery -u -o out/ -c "in1/;bg1/;New Collection"

	- Add pictures to existing collection (collection with same title must already exist, else it will be created as new)
	  static_gallery -u -o out/ -c "in2/;-;Collection 01"

	- Add backgrounds to existing collection (collection with same title must already exist, else it will be created as new)
	  static_gallery -u -o out/ -c "-;bg2/;Collection 01"

	USAGE:
		static_gallery [FLAGS] [OPTIONS] --output <output-dir> --template <template-dir>

	FLAGS:
		-r, --remove-output
				Whether to clear the output irectory

		-a, --archive
				Whether to create an archive (downloadable zip-file) with the original pictures

		-h, --help
				Prints help information

			--image-name-titles
				When set to true the image names (without extensions) are used as picture titles

		-u, --update
				Update gallery (add new pictures to existing gallery) overwrites pictures with the same file name in the
				same collection
		-V, --version
				Prints version information

		-v, --verbose
				Increases the log level. By default only errors are shown. Levels: Error, Warning, Info, Debug


	OPTIONS:
			--background-size <background-size>
				The size of the backgroun picture versions [default: 2560x1440]

		-c, --collection <collections>...
				Collection input as "[input directory];[background directory],[collection title]". Examples: "in/;bg/;Col
				1", "in/;-;Col 2"
			--display-size <display-size>
				The size of the display picture versions [default: 2560x1440]

			--jpeg-quality <jpeg-quality>
				Quality of the output images 1-100 [default: 75]

		-o, --output <output-dir>
				The output directory for the generated gallery

			--resize-method <resize-method>
				Image resize method. Valid methods: "lanczos3", "gaussian", "nearest", "cubic", "linear" [default: lanczos3]

		-p, --template <template-dir>
				The directory of the template to be used for the gallery

			--threads <threads>
				Number of concurrent threads to use for image resizing. If set to 0 it uses the number of available logical
				cores [default: 0]
			--thumb-size <tumb-size>
				The size of the small picture versions (thumbnails) [default: 960x540]

## Available templates

Currently there is only one template available (which is the reason for this project).

### Template "hauer"

This template should simulate pictures lying on a background. The background scrolls from one picture to the next while staying at the same position. When clicking on the picture it is opened in a larger view and can be downloaded via a link.

IE is not supported by this template.

The configuration is done in the global galleryConfig object:

```javascript
window.galleryConfig = {
	// If autoStart is set to true, the gallery is created as soon as the script is loaded
	// If autostart is set to false, the gallery is loaded only when window.galleryInit([config]) is called. This can be used to load (parts of) the configuration asynchronously
	"autoStart": true,

	// If preloadThumbs is set to true, the loadingIndicator is shown while the picture thumbnails are loaded, then the gallery is shown
	// If preloadThumbs is set to a number x, the first x backgrounds will be preloaded
	"preloadThumbs": 6,
	// If preloadBackgrounds is set to true, the loadingIndicator is shown while the background pictures are loaded, then the gallery is shown
	// If preloadBackgrounds is set to a number x, the first x backgrounds will be preloaded
	"preloadBackgrounds": 2,

	// Configuration options for the polaroid style thumbnails
	"thumbs": {
		// Randomized rotation maximum
		"maxRotation": 50,
		// If randomizePosition is not set, the thumbnail pictures are distributed evenly
		// If hoverRevert is set to true, the thumbnail will return to its regular position on hover
		// Amount and unit are split to allow calculation without parsing
		// ! Remember to set the #content padding to a higher value if you increase the amount here to avoid pictures being cut off
		"randomizePosition": {
			"amount": 7,
			"unit": "vmin",
			"hoverRevert": true,
		}
	},

	// Configuration for the main background
	"background": {
		// If overscroll is set to true, all background images will be shown and can be scrolled down to
		// If overscroll is set to false, only background images are shown for the area there thumbnails are shown, but at least one full window size
		"overscroll": false
	},


	// Configuration options for the full display
	"display": {
		// Whether to show a download button for single images linking to the full picture
		"download": true
	},
};
```

The gallery data will be embedded in the index.html and replace the `[]` between the patterns `/*{{BEGIN:data*/` and `/*END:data}}*/`

#### Known issues

- Old browsers like IE are not supported but there is no message, just a black screen
- If not enough background pictures are provided, a black background is reached at some point
