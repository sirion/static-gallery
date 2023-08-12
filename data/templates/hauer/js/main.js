(function () {
	"use strict";

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
		const key = config.collection_keys[0];
		const collection = config.collections[key];
		const backgrounds = collection.backgrounds;

		let images = [];
		if (config.preloadBackgrounds) {
			if (isFinite(config.preloadBackgrounds) && config.preloadBackgrounds > 0) {
				images = images.concat(backgrounds.slice(0, config.preloadBackgrounds));
			} else {
				images = images.concat(backgrounds);
			}
			images = images.map(pic => "p/" + pic.path + ".bg." + config.extension);
		}

		if (config.preloadThumbs) {
			let thumbs = [];
			if (isFinite(config.preloadThumbs) && config.preloadThumbs > 0) {
				thumbs = thumbs.concat(collection.pictures.slice(0, config.preloadThumbs));
			} else {
				thumbs = thumbs.concat(collection.pictures);
			}
			thumbs = thumbs.map(pic => "p/" + pic.path + ".thumb." + config.extension);
			images = images.concat(thumbs);
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

	function hash(key, value = null) {
		const parts = window.location.hash.split("|").filter(p => !!p);
		const hashes = parts.reduce((prev, cur) => {
			const [ key, ...values ] = cur.split("=");
			if (key && values.length > 0) {
				prev[key] = values.join("=");
			}
			return prev;
		}, {});

		if (value === null) {
			// Getter
			return hashes[key];
		} else {
			// Setter
			if (value === "") {
				delete hashes[key];
			} else {
				hashes[key] = value
			}
			window.location.hash = "|" + Object.entries(hashes).map(e => `${e[0]}=${e[1]}`).join("|") + "|";
		}
	}

	function galleryShow(config) {
		const bg = document.querySelector("#pictureDisplay");
		const img = document.querySelector("#pictureDisplay > img");
		const vid = document.querySelector("#pictureDisplay > video");
		const fullsizeLink = document.querySelector("#pictureDisplay > a");

		fullsizeLink.addEventListener("click", e => e.stopPropagation());
		let activeThumb = null;

		let curPictureIndex = 0;
		let curCollectionKey = 0;
		let showing = false;
		let showingVideo = false;

		function showPicture(collection, index, event) {
			const isVideo = !!collection.pictures[index].video;
			showingVideo = isVideo;
			const path = collection.pictures[index].path;
			const picture = isVideo ? "v/" + path : "p/" + path;
			curPictureIndex = index;
			showing = true;

			hash("i", index);

			if (event) {
				activeThumb = event.target;
				activeThumb.classList.add("active");
			}

			bg.style.opacity = 1;
			bg.style.zIndex = 10;
			
			if (isVideo) {
				img.style.display = "none";
				vid.style.display = "block";
				vid.src = picture + "." + config.videoExtension;
				vid.play();

				fullsizeLink.style.display = "none";
			} else {
				img.style.display = "block";
				vid.style.display = "none";
				img.src = picture + ".disp." + config.extension;

				fullsizeLink.style.display = "";
			}

			fullsizeLink.href = picture + "." + config.extension;
		}

		function showNextPicture() {
			const collection = config.collections[curCollectionKey];
			curPictureIndex++;
			if (!collection.pictures[curPictureIndex]) {
				showFirstPicture();
			} else {
				showPicture(collection, curPictureIndex)
			}
		}

		function showPreviousPicture() {
			const collection = config.collections[curCollectionKey];
			curPictureIndex--;
			if (!collection.pictures[curPictureIndex]) {
				showLastPicture();
			} else {
				showPicture(collection, curPictureIndex)
			}
		}

		function showFirstPicture() {
			const collection = config.collections[curCollectionKey];
			curPictureIndex = 0;
			showPicture(collection, curPictureIndex)
		}

		function showLastPicture() {
			const collection = config.collections[curCollectionKey];
			curPictureIndex = collection.pictures.length - 1;
			showPicture(collection, curPictureIndex)
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

				hash("i", "");
				if (showingVideo) {
					vid.pause();
				}
			};
			bg.addEventListener("transitionend", gone);
			bg.style.opacity = 0;
		}

		function clear(element) {
			while (element.childNodes.length > 0) {
				element.removeChild(element.childNodes[0]);
			}
		}

		function showCollection(key, config) {
			curCollectionKey = key;
			hash("c", key);

			bg.addEventListener("click", e => {
				if (e.target !== img && e.target !== vid) {
					hidePicture();
				}
			});

			const bgContainer = document.querySelector("#background");
			clear(bgContainer);

			const collection = config.collections[key];
			
			collection.backgrounds.forEach(pic => {
				const bg = document.createElement("div");
				bg.classList.add("p", "bg");
				bg.style.backgroundImage = "url('p/" + pic.path + ".bg." + config.extension + "')";
				bgContainer.appendChild(bg);
			});



			const contentContainer = document.querySelector("#content");
			clear(contentContainer);
			collection.pictures.forEach((pic, index) => {
				const title = pic.title;
				const path = pic.video ? "v/" + pic.path : "p/" + pic.path;


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
				thumb.addEventListener("click", showPicture.bind(null, collection, index));

				if (config.thumbs && config.thumbs.randomizePosition) {
					thumb.style.top = (config.thumbs.randomizePosition.amount - (Math.random() * config.thumbs.randomizePosition.amount * 2)) + config.thumbs.randomizePosition.unit;
					thumb.style.left = (config.thumbs.randomizePosition.amount - (Math.random() * config.thumbs.randomizePosition.amount * 2)) + config.thumbs.randomizePosition.unit;

					if (config.thumbs.randomizePosition.hoverRevert) {
						thumb.classList.add("revert");
					}
				}

				contentContainer.appendChild(thumb);
			});
			
			if (collection.backgrounds.length > 0 && (!config.background || !config.background.overscroll)) {
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

		let collectionName = hash("c")
		let collectionIndex = collectionName ? config.collection_keys.indexOf(collectionName) : -1;
		if (collectionIndex == -1) {
			collectionName = config.collection_keys[0];
			collectionIndex = 0;
		}
		showCollection(config.collection_keys[collectionIndex], config);

		const pictureIndex = hash("i");
		if (pictureIndex !== undefined && pictureIndex !== "") {
			showPicture(config.collections[collectionName], pictureIndex);
		}


		if (config.collection_keys.length > 1) {
			// TODO: Preloader for the other collections

			// Show Menu to switch collections
			const collectionSwitch = document.querySelector("#collectionSwitch");
			collectionSwitch.style.display = "block";

			const blackScreen = document.querySelector("#blackScreen");
			const collectionMenu = document.querySelector("#collectionSwitch > .menu");
			const collectionButton = document.querySelector("#collectionSwitch > .button");

			let switchCollectionKey = null;
			function switchCollection() {
				if (switchCollectionKey === null) {
					blackScreen.style.zIndex = "";
					blackScreen.removeEventListener("transitionend", switchCollection);
				} else {
					showCollection(switchCollectionKey, config);
					document.documentElement.scrollTo(0, 0)

					switchCollectionKey = null;
					setTimeout(() => {
						blackScreen.classList.remove("active");
					}, 500);
				}
			}

			config.collection_keys.forEach(key => {
				const collection = config.collections[key];
				const entry = document.createElement("div");
				entry.textContent = collection.title;
				entry.addEventListener("click", function(key) {
					// TODO: Fade to/from black
					switchCollectionKey = key;
					blackScreen.addEventListener("transitionend", switchCollection);
					blackScreen.style.zIndex = "10";
					blackScreen.classList.add("active");
				}.bind(null, collection.name))

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