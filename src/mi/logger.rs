// #![allow(dead_code)]

use lazy_static::lazy_static;
use std::io::Write;

static mut LOG_LEVEL: u8 = 0;

pub const LOGLEVEL_DEBUG: u8 = 4;
pub const LOGLEVEL_INFO: u8 = 3;
pub const LOGLEVEL_WARN: u8 = 2;
pub const LOGLEVEL_ERROR: u8 = 1;
// pub const LOGLEVEL_SILENT: u8 = 0;

lazy_static! {
	pub static ref _MUTEX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
	pub static ref LEVEL_NAMES: Vec<String> = vec![
		String::from(""),
		String::from("[ Error ] "),
		String::from("[Warning] "),
		String::from("[ Info  ] "),
		String::from("[ Debug ] ")
	];
}

pub fn set_level(level: u8) {
	unsafe {
		LOG_LEVEL = level;
	}
}

pub fn get_level() -> u8 {
	unsafe { LOG_LEVEL }
}

pub fn flush() {
	std::io::stdout().flush().ok();
}


pub fn debug(message: String) {
	log(LOGLEVEL_DEBUG, message);
}
pub fn debugln(message: String) {
	logln(LOGLEVEL_DEBUG, message);
}

pub fn info(message: String) {
	log(LOGLEVEL_INFO, message);
}
pub fn infoln(message: String) {
	logln(LOGLEVEL_INFO, message);
}

pub fn warn(message: String) {
	log(LOGLEVEL_WARN, message);
}
pub fn warnln(message: String) {
	logln(LOGLEVEL_WARN, message);
}

pub fn error(message: String) {
	log(LOGLEVEL_ERROR, message);
}
pub fn errorln(message: String) {
	logln(LOGLEVEL_ERROR, message);
}

pub fn log(level: u8, message: String) {
	_log(false, level, message);
}

pub fn logln(level: u8, message: String) {
	_log(true, level, message);
}

fn _log(ln: bool, level: u8, message: String) {
	if level > 0 && level <= LOGLEVEL_DEBUG && get_level() >= level {
		let guard =_MUTEX.lock();

		if level == LOGLEVEL_ERROR {
			if ln {
				eprintln!("{}{}", LEVEL_NAMES[level as usize], message);
			} else {
				eprint!("{}{}", LEVEL_NAMES[level as usize], message);
			}
			std::io::stderr().flush().ok();
		} else {
			if ln {
				println!("{}{}", LEVEL_NAMES[level as usize], message);
			} else {
				print!("{}{}", LEVEL_NAMES[level as usize], message);
			}
			std::io::stdout().flush().ok();
		}
		drop(guard);
	}
}


// #[macro_export]
// macro_rules! debug {
// 	($($arg:tt)*) => {
// 		crate::logger::debug(format!($($arg)*));
// 	};
// }

// #[macro_export]
// macro_rules! debugln {
// 	($($arg:tt)*) => {
// 		crate::logger::debugln(format!($($arg)*));
// 	};
// }

// #[macro_export]
// macro_rules! info {
// 	($($arg:tt)*) => {
// 		crate::logger::info(format!($($arg)*));
// 	};
// }
// #[macro_export]
// macro_rules! infoln {
// 	($($arg:tt)*) => {
// 		crate::logger::infoln(format!($($arg)*));
// 	};
// }


// #[macro_export]
// macro_rules! warn {
// 	($($arg:tt)*) => {
// 		crate::logger::warn(format!($($arg)*));
// 	};
// }
// #[macro_export]
// macro_rules! warnln {
// 	($($arg:tt)*) => {
// 		crate::logger::warnln(format!($($arg)*));
// 	};
// }


// #[macro_export]
// macro_rules! error {
// 	($($arg:tt)*) => {
// 		crate::logger::error(format!($($arg)*));
// 	};
// }
// #[macro_export]
// macro_rules! errorln {
// 	($($arg:tt)*) => {
// 		crate::logger::errorln(format!($($arg)*));
// 	};
// }

