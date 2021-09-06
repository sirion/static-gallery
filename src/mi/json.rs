
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum JsonType {
	Object,
	Array,
	String,
	Number,
	Bool,
	Null,
	/// Incomplete type is a placeholder that will be changed in the next operation. It is ignored when creating JSON string.
	Incomplete,
}

impl std::fmt::Display for JsonType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", match self {
			JsonType::Array => "array",
			JsonType::Bool => "bool",
			JsonType::Number => "number",
			JsonType::Object => "object",
			JsonType::String => "string",
			JsonType::Null => "null",
			JsonType::Incomplete => "Incomplete (Invalid)",
		})
	}
}

pub struct JsonData {
	t: JsonType,
	parent: Option<Rc<RefCell<JsonData>>>,
	object_properties: HashMap<String, Rc<RefCell<JsonData>>>,
	array_values: Vec<Rc<RefCell<JsonData>>>,
	str_value: String,
}


impl JsonData {
	pub fn new(t: JsonType) -> JsonData {
		JsonData{
			t,
			parent: Option::None,
			array_values: Vec::new(),
			object_properties: HashMap::new(),
			str_value: String::default(),
		}
	}

	pub fn stringify(&self) -> String {
		let mut buf: Vec<String> = vec![];

		match self.t {
			JsonType::Array => {
				buf.push(String::from("["));
				if self.array_values.len() > 0 {
					for v in &self.array_values {
						buf.push(v.borrow().stringify());
						buf.push(String::from(","));
					}
					buf.pop();
				}
				buf.push(String::from("]"));
			},
			JsonType::Bool => {
				buf.push(self.str_value.clone());
			},
			JsonType::Number => {
				buf.push(self.str_value.clone());
			},
			JsonType::Object => {
				buf.push(String::from("{"));
				if self.object_properties.len() > 0 {
					for (k, v) in &self.object_properties {
						buf.push(String::from("\""));
						buf.push(k.clone());
						buf.push(String::from("\":"));

						buf.push(v.borrow().stringify());
						buf.push(String::from(","));
					}
					buf.pop();
				}
				buf.push(String::from("}"));
			},
			JsonType::String => {
				buf.push(String::from("\""));
				buf.push(self.str_value.replace("\"", "\\\""));
				buf.push(String::from("\""));
			},
			JsonType::Null => {
				buf.push(String::from("null"));
			},
			JsonType::Incomplete => {},
		}

		buf.join("")
	}
}



pub struct Json {
	root: Rc<RefCell<JsonData>>,
	current: Rc<RefCell<JsonData>>,
}

#[allow(dead_code)]
impl Json {
	pub fn new(t: JsonType) -> Json {
		let root = Rc::new(RefCell::new(JsonData::new(t)));
		let current = root.clone();

		Json{ root, current }
	}


	pub fn start(&mut self, t: JsonType) -> &mut Json {

		let mut c = self.current.borrow_mut();
		let mut ct = c.t;
		let mut new = true;

		let node: Rc<RefCell<JsonData>>;
		if ct == JsonType::Incomplete {
			node = Rc::clone(&self.current);
			c.t = t;
			ct = t;
			new = false;
		} else {
			node = Rc::new(RefCell::new(JsonData::new(t)));
		}


		drop(c);

		if new && ct == JsonType::Array {
			// Add to array
			let mut c = self.current.borrow_mut();
			c.array_values.push(Rc::clone(&node));
			drop(c);

			let mut node_content = node.borrow_mut();
			(*node_content).parent.replace(Rc::clone(&self.current));
		} else if new {
			panic!("Not allowed to start {} in {}", t, ct);
		}

		self.current = node;

		self
	}

	pub fn end(&mut self) -> &mut Json {
		let current = self.current.clone();
		let c = current.borrow();

		let p_option = &c.parent;
		match p_option {
			None => {
				eprintln!("end not possible");
			},
			Some(p) => {
				self.current = Rc::clone(p);
			},
		};

		self
	}


	pub fn array(&mut self) -> &mut Json {
		self.start(JsonType::Array)
	}

	pub fn object(&mut self) -> &mut Json {
		self.start(JsonType::Object)
	}

	pub fn null(&mut self) -> &mut Json {
		self.start(JsonType::Null)
	}

	pub fn string<T: ToString>(&mut self, value: T) -> &mut Json {
		self.start(JsonType::String);

		let mut c = self.current.borrow_mut();
		c.str_value = value.to_string();
		drop(c);

		self.end()
	}

	pub fn start_property<T: ToString>(&mut self, name: T, t: JsonType) -> &mut Json {
		let mut c = self.current.borrow_mut();
		if c.t != JsonType::Object {
			panic!("Not allowed to add property to a {} ", c.t);
		}

		let node = Rc::new(RefCell::new(JsonData::new(t)));

		c.object_properties.insert(name.to_string(), Rc::clone(&node));
		drop(c);

		let mut node_content = node.borrow_mut();
		(*node_content).parent.replace(Rc::clone(&self.current));
		drop(node_content);

		self.current = node;

		self
	}

	pub fn value<T: ToString>(&mut self, value: T) -> &mut Json {
		let mut c = self.current.borrow_mut();

		if c.t == JsonType::Incomplete {
			// Switch type
			c.t = JsonType::String;
		}

		if c.t == JsonType::Bool {
			if !value.to_string().eq("true") && !value.to_string().eq("false") {
				panic!("Cannot set bool property to \"{}\"", value.to_string());
			}
		} else if c.t == JsonType::Number {
			let value_int = value.to_string().trim().parse::<i64>();
			let value_float = value.to_string().trim().parse::<f64>();

			match value_float {
				Ok(_) => {},
				Err(_) => {
					match value_int {
						Ok(_) => {},
						Err(_) => {
							panic!("Cannot set number property to \"{}\"", value.to_string());
						}
					};
				}
			};

		} else if c.t != JsonType::String {
			panic!("Cannot set value of a {} to \"{}\"", c.t, value.to_string());
		}

		c.str_value = value.to_string();

		drop(c);
		self
	}

	pub fn stringify(&self) -> String {
		self.root.borrow().stringify()
	}
}


impl std::fmt::Display for Json {
 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 		f.write_str(self.stringify().as_str())
 	}
}
