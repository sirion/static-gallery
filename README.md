# static-gallery
## Create static image galleries for the web

Use the `bin/generate-gallery` executable to generate a gallery that can be served on webserver using only static resources.

the `bin` folder contains compiled executables for linux, mac and windows.


## `generate-gallery` command

Usage:

	generate-gallery -output foldername -template path/to/template path/to/collection(s)

The path must be a folder directly containing the background images and a folder containing images for each collection.


The following options must be provided:

	-output string
		Directory where the generated gallery is written to
	-template string
		Directory containing the template


The following options can be changed:

	-backgroundSize string
		Maximum size of display pictures (default "2560x1440")
	-displaySize string
		Maximum size of display pictures (default "2560x1440")
	-thumbSize string
		Maximum size of thumbnail pictures (default "960x540")

	-optimize
		Embed JavaScript and CSS into the HTML file and remove comments


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
		// Whether to show a download button
		"download": true
	},

	// The collections will be generated
	collections: /*{{BEGIN:collections*/ [] /*END:collections}}*/
};
```

#### Known issues:

 - Old browsers like IE are not supported but there is no message, just a black screen
 - If not enough background pictures are provided, a black background is reached at some point
