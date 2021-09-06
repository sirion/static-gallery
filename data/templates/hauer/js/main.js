(function () {
	"use strict";

	// TODO: Deep Links to collection and picture

	window.galleryInit = function galleryInit(config) {
		if (config.preloadThumbs || config.preloadBackgrounds) {
			const loadingScreenBg = document.querySelector("#loading");
			if (loadingScreenBg) {
				loadingScreenBg.style.display = "flex";
			}
			galleryPreload(config);
		} else {
			galleryShow(config);
		}
	}


	function galleryPreload(config) {
		let images = [];
		if (config.preloadBackgrounds) {
			const backgrounds = (isFinite(config.preloadBackgrounds) && config.preloadBackgrounds > 0) ? config.collections[0].backgrounds.slice(0, config.preloadBackgrounds) : config.collections[0].backgrounds;
			images = images.concat(backgrounds);
		}
		if (config.preloadThumbs) {
			const pictures = (isFinite(config.preloadThumbs) && config.preloadThumbs > 0) ? config.collections[0].pictures.slice(0, config.preloadThumbs) : config.collections[0].pictures;
			images = images.concat(pictures.map(pic => {
				return (pic.thumb ? pic.thumb : pic.picture);
			}));
		}

		preloadImages(images).then(galleryShow.bind(null, config));
	}

	function loadImage(image, loadingProgressCallback) {
		return new Promise((resolve) => {
			var img = new Image();
			img.addEventListener("load", resolve);
			img.addEventListener("error", resolve);
			img.src = image;
		}).then(loadingProgressCallback);
	}

	function preloadImages(images) {
		let chain = Promise.resolve();
		images.forEach((image, i) => {
			chain = chain.then(() => loadImage(image, loadingProgressCallback.bind(null, images.length, i + 1)));
		});
		return chain;
	}

	function loadingProgressCallback(numAll, numLoaded) {
		const loadingScreenBg = document.querySelector("#loading");
		const loadingScreenIndicator = document.querySelector("#loading .indicator");

		loadingScreenBg.style.display = "flex";
		loadingScreenIndicator.style.width = (((numLoaded) / numAll) * 100) + "%";
		if (numLoaded === numAll) {
			loadingScreenBg.style.opacity = 0;
			loadingScreenBg.addEventListener("transitionend", () => {
				loadingScreenBg.style.display = "none";
			});
		}
	}

	function galleryShow(config) {
		const bg = document.querySelector("#pictureDisplay");
		const img = document.querySelector("#pictureDisplay > img");
		const fullsizeLink = document.querySelector("#pictureDisplay > a");

		fullsizeLink.addEventListener("click", e => e.stopPropagation());
		let activeThumb = null;

		let curPictureIndex = 0;
		let curCollectionIndex = 0;
		let showing = false;

		function showPicture(picture, index, config, event) {
			curPictureIndex = index;
			showing = true;

			if (event) {
				activeThumb = event.target;
				activeThumb.classList.add("active");
			}

			bg.style.opacity = 1;
			bg.style.zIndex = 10;
			img.src = picture + ".disp." + config.extension;

			fullsizeLink.href = picture + "." + config.extension;
		}

		function showNextPicture() {
			const collection = config.collections[curCollectionIndex].pictures;
			curPictureIndex++;
			if (!collection[curPictureIndex]) {
				showFirstPicture();
			} else {
				showPicture(collection[curPictureIndex], curPictureIndex)
			}
		}

		function showPreviousPicture() {
			const collection = config.collections[curCollectionIndex].pictures;
			curPictureIndex--;
			if (!collection[curPictureIndex]) {
				showLastPicture();
			} else {
				showPicture(collection[curPictureIndex], curPictureIndex)
			}
		}

		function showFirstPicture() {
			const collection = config.collections[curCollectionIndex].pictures;
			curPictureIndex = 0;
			showPicture(collection[curPictureIndex], curPictureIndex)
		}

		function showLastPicture() {
			const collection = config.collections[curCollectionIndex].pictures;
			curPictureIndex = collection.length - 1;
			showPicture(collection[curPictureIndex], curPictureIndex)
		}

		function hidePicture() {
			showing = false;
			const gone = function () {
				bg.style.zIndex = 0;
				img.src = "";

				if (activeThumb) {
					activeThumb.classList.remove("active");
					activeThumb = null;
				}
				bg.removeEventListener("transitionend", gone);
			};
			bg.addEventListener("transitionend", gone);
			bg.style.opacity = 0;
		}

		function clear(element) {
			while (element.childNodes.length > 0) {
				element.removeChild(element.childNodes[0]);
			}
		}

		function showCollection(num, config) {
			curCollectionIndex = num;

			bg.addEventListener("click", hidePicture);

			const bgContainer = document.querySelector("#background");
			clear(bgContainer);
			config.collections[num].backgrounds.forEach(pic => {
				const bg = document.createElement("div");
				bg.classList.add("p", "bg");
				bg.style.backgroundImage = "url('" + pic.path + "." + config.extension + "')";
				bgContainer.appendChild(bg);
			});



			const contentContainer = document.querySelector("#content");
			clear(contentContainer);
			config.collections[num].pictures.forEach((pic, index) => {
				const title = pic.title;
				const path = pic.path;

				const thumb = document.createElement("div");
				thumb.classList.add("p", "thumb");
				if (title) {
					thumb.title = title;
					thumb.classList.add("titled");
					const thumbTitle = document.createElement("span");
					thumbTitle.textContent = title;
					thumb.appendChild(thumbTitle);
				}
				thumb.style.backgroundImage =
					"url('" + path + ".thumb." + config.extension + "')";
				thumb.style.transform =
					"rotate(" +
					Math.round(
						0 -
						config.thumbs.maxRotation +
						2 * config.thumbs.maxRotation * Math.random()
					) +
					"deg)";
				thumb.addEventListener("click", showPicture.bind(null, path, index, config));

				if (config.thumbs && config.thumbs.randomizePosition) {
					thumb.style.top = (config.thumbs.randomizePosition.amount - (Math.random() * config.thumbs.randomizePosition.amount * 2)) + config.thumbs.randomizePosition.unit;
					thumb.style.left = (config.thumbs.randomizePosition.amount - (Math.random() * config.thumbs.randomizePosition.amount * 2)) + config.thumbs.randomizePosition.unit;

					if (config.thumbs.randomizePosition.hoverRevert) {
						thumb.classList.add("revert");
					}
				}

				contentContainer.appendChild(thumb);
			});

			if (!config.background || !config.background.overscroll) {
				const onResize = () => {
					bgContainer.style.height = Math.max(contentContainer.scrollHeight, contentContainer.offsetHeight) + "px";
				};
				window.addEventListener("resize", onResize);
				onResize();
			}

			// TODO: Links
		}

		// Keyboard Navigation
		document.body.addEventListener("keydown", event => {
			if (!showing) {
				return;
			}
			event.preventDefault();

			switch (event.key) {
				case "ArrowRight":
				case "ArrowDown":
				case "PageDown":
					showNextPicture();
					break;

				case "ArrowLeft":
				case "ArrowUp":
				case "PageUp":
					showPreviousPicture();
					break;

				case "Home":
					showFirstPicture();
					break;

				case "End":
					showLastPicture();
					break;

				case "Escape":
					hidePicture();
					break;
			}
		});

		showCollection(0, config);

		if (config.collections.length > 1) {
			// TODO: Preloader for the other collections

			// Show Menu to switch collections
			const collectionSwitch = document.querySelector("#collectionSwitch");
			collectionSwitch.style.display = "block";

			const blackScreen = document.querySelector("#blackScreen");
			const collectionMenu = document.querySelector("#collectionSwitch > .menu");
			const collectionButton = document.querySelector("#collectionSwitch > .button");

			let switchCollectionNum = null;
			function switchCollection() {
				if (switchCollectionNum === null) {
					blackScreen.style.zIndex = "";
					blackScreen.removeEventListener("transitionend", switchCollection);
				} else {
					showCollection(switchCollectionNum, config);
					document.documentElement.scrollTo(0, 0)

					switchCollectionNum = null;
					setTimeout(() => {
						blackScreen.classList.remove("active");
					}, 500);
				}
			}

			config.collections.forEach((collection, num) => {
				const entry = document.createElement("div");
				entry.textContent = collection.title;
				entry.addEventListener("click", function(num) {
					// TODO: Fade to/from black
					switchCollectionNum = num;
					blackScreen.addEventListener("transitionend", switchCollection);
					blackScreen.style.zIndex = "10";
					blackScreen.classList.add("active");
				}.bind(null, num))

				collectionMenu.appendChild(entry)
			});

			collectionButton.addEventListener("click", function(event) {
				// TODO: Fade to/from black
				collectionSwitch.classList.add("open");
				event.stopPropagation();
			});

			document.addEventListener("click", () => {
				collectionSwitch.classList.remove("open");
			});
		}


		if (config.archives) {
			const footer = document.querySelector("#footer");


			for (const key in config.archives) {
				const archive = document.createElement("div");
				archive.classList.add("archive");

				const link  = document.createElement("a");
				link.href = config.archives[key];

				if (key == "_full_") {
					link.textContent = "Download Gallery Archive"; // TODO: I18N
				} else {
					link.textContent = key;
				}

				archive.append(link);
				footer.append(archive);
			}
		}

		// TODO: Links
	}

})();