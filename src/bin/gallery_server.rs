use mi::http::methods::*;
use mi::http::{lookup_status_str, Request, Response, Server};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

fn main() {
	mi::logger::set_level(mi::logger::LOGLEVEL_DEBUG);

	let mut server = Server::new();
	server.log_access = Arc::new(Mutex::new(std::io::stdout()));

	// let frontent_path = std::path::PathBuf::from("data/test/output/");
	// let mut file_handler = FileHandler::new("/", frontent_path);
	// file_handler.list_dirs = true;
	// server.handler(Arc::new(file_handler));

	server.handle(|_| true, handle_api);
	server.listen(8080).unwrap();
}

fn handle_api(req: &Request, res: Response) {
	let mut parts: VecDeque<&str> = req.uri.split('/').collect();

	if parts.is_empty() || !parts[0].is_empty() {
		// Weird...
		respond_error(res, 400, format!("Invalid Request: {}", req.uri));
	} else {
		parts.remove(0);
		respond_api(req, res, parts);
	}
}

//
//
// NOT YET IMPLEMENTED
//
//

fn respond_nyi(req: &Request, res: Response) {
	respond_error(res, 599, format!("Not Yet Implemented: {}", req.uri))
}

//
//
//
// General response functions
//
//
//

fn respond_error<S: AsRef<str>>(mut res: Response, status: u16, message: S) {
	res.status_code = status;
	let m = match message.as_ref().is_empty() {
		true => lookup_status_str(status),
		false => message.as_ref(),
	};
	let _ = write!(res, "{{ \"error:\": \"{}\" }}", m);
}

fn respond_json<J: ?Sized + serde::Serialize>(mut res: Response, data: &J) {
	res.headers.set("Content-Type", "application/json");
	match serde_json::to_string_pretty(data) {
		Ok(json) => {
			let _ = res.write(json);
		}
		Err(e) => respond_error(res, 500, e.to_string()),
	};
}

//
//
//
// API response functions
//
//
//

fn respond_api(req: &Request, mut res: Response, mut parts: VecDeque<&str>) {
	if parts.is_empty() || parts[0].is_empty() {
		res.headers.set("Content-Type", "application/json");
		let _ = res.write(r#"{ "name": "Gallery API", "version": "0.1.0" }"#);
		return;
	}

	let cmd = parts.pop_front();

	match cmd {
		Some("galleries") => {
			let g_id = parts.pop_front();
			let c_id = parts.pop_front();
			let p_id = parts.pop_front();

			if !parts.is_empty() {
				respond_error(res, 404, "Invalid API Endpoint");
			} else {
				respond_api_gallery(req, res, g_id, c_id, p_id);
			}
		}

		Some("login") => {
			respond_nyi(req, res);
		}
		Some("logout") => {
			respond_nyi(req, res);
		}

		Some(_) | None => respond_error(res, 404, format!("Not found: {}", req.uri)),
	}
}

fn respond_api_gallery(
	req: &Request,
	res: Response,
	gid: Option<&str>,
	cid: Option<&str>,
	pid: Option<&str>,
) {
	let user = match User::get(req) {
		Some(u) => u,
		None => {
			return respond_error(res, 401, "");
		}
	};

	// let params = req.get_query_parameters();
	// let action = params.get("action");

	if gid == None {
		// Level: All galleries.
		match &*req.method {
			GET => {
				// List all gallery IDs for current user
				return respond_json(res, &get_galleries(&user));
			}
			PUT => {
				// Create a new gallery with the data given in the request body
				return respond_nyi(req, res);
			}
			_ => {
				return respond_error(res, 400, "Invalid API Invocation");
			}
		}
	} else if cid == None {
		// Level: Specific gallery
		let gallery_id = gid.unwrap();
		let gallery = match Gallery::get(gallery_id) {
			None => {
				let message = format!("Gallery with ID \"{}\" not found", gallery_id);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		match &*req.method {
			GET => {
				// Return the gallery information including all collection IDs
				return respond_json(res, &gallery);
			}
			POST => {
				// TODO: Update the gallery with the given data in the request body
				return respond_nyi(req, res);
			}
			PUT => {
				// TODO: Add a new collection to the gallery with the given data in the request body
				return respond_nyi(req, res);
			}
			_ => {
				return respond_error(res, 400, "Invalid API Invocation");
			}
		}
	} else if pid == None {
		// Level: Specific Collection

		let gallery_id = gid.unwrap();
		let gallery = match Gallery::get(gallery_id) {
			None => {
				let message = format!("Gallery with ID \"{}\" not found", gallery_id);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		let collection_id = cid.unwrap();
		let collection = match gallery.get_collection(collection_id) {
			None => {
				let message = format!(
					"Collection with ID \"{}\" not found in gallery \"{}\"",
					collection_id, gallery_id
				);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		match &*req.method {
			GET => {
				// Return the Collection information including all picture IDs
				return respond_json(res, &collection);
			}
			POST => {
				// TODO: Update the collection with the given data in the request body
				return respond_nyi(req, res);
			}
			PUT => {
				// TODO: Add a new picture to the gallery with the given data in the request body
				return respond_nyi(req, res);
			}
			_ => {
				return respond_error(res, 400, "Invalid API Invocation");
			}
		}
	} else {
		// Level: Specific Picture

		let gallery_id = gid.unwrap();
		let gallery = match Gallery::get(gallery_id) {
			None => {
				let message = format!("Gallery with ID \"{}\" not found", gallery_id);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		let collection_id = cid.unwrap();
		let collection = match gallery.get_collection(collection_id) {
			None => {
				let message = format!(
					"Collection with ID \"{}\" not found in gallery \"{}\"",
					collection_id, gallery_id
				);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		let picture_id = pid.unwrap();
		let picture = match collection.get_picture(picture_id) {
			None => {
				let message = format!(
					"Picture with ID \"{}\" not found in collection \"{}\" of gallery \"{}\"",
					picture_id, collection_id, gallery_id
				);
				return respond_error(res, 404, message);
			}
			Some(g) => g,
		};

		match &*req.method {
			GET => {
				// Return the Picture information
				return respond_json(res, &picture);
			}
			POST => {
				// TODO: Update the picture with the given data in the request body
				return respond_nyi(req, res);
			}
			_ => {
				return respond_error(res, 400, "Invalid API Invocation");
			}
		}
	}
}

//
//
//
// User
//
//
//

struct User {}

impl User {
	fn get(_req: &Request) -> Option<User> {
		// TODO: Find valid session id in cookies or return none
		Some(User {})
	}
}

//
//
//
// Gallery
//
//
//

#[derive(serde::Serialize)]
struct Gallery<'a> {
	id: &'a str,
}

impl<'a> Gallery<'a> {
	pub fn get(gallery_id: &str) -> Option<Gallery> {
		match gallery_id {
			"test" => Some(Gallery { id: gallery_id }),
			_ => None,
		}
	}

	pub fn get_collection(&self, collection_id: &'a str) -> Option<Collection<'a>> {
		match collection_id {
			"test" => Some(Collection { id: collection_id }),
			_ => None,
		}
	}
}

//
//
//
// Collection
//
//
//

#[derive(serde::Serialize)]
struct Collection<'a> {
	id: &'a str,
}

impl<'a> Collection<'a> {
	pub fn get_picture(&self, picture_id: &'a str) -> Option<Picture<'a>> {
		match picture_id {
			"test" => Some(Picture { id: picture_id }),
			_ => None,
		}
	}
}

//
//
//
// Picture
//
//
//

#[derive(serde::Serialize)]
struct Picture<'a> {
	id: &'a str,
}

impl<'a> Picture<'a> {}

//
//
//
// Misc Utility Functions
//
//
//

fn get_galleries(_u: &User) -> Vec<&str> {
	vec!["test"]
}
