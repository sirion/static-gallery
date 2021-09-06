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

#[macro_export]
macro_rules! debug {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_DEBUG {
			let guard = crate::mi::logger::_MUTEX.lock();
			print!("[Debug] ");
			print!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
#[macro_export]
macro_rules! debugln {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_DEBUG {
			let guard = crate::mi::logger::_MUTEX.lock();
			print!("[Debug] ");
			println!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
// pub fn debug(s: &str) {
// 	if LOG_LEVEL >= LOGLEVEL_DEBUG {
// 		println!("[Debug] {}", s);
// 	}
// }

#[macro_export]
macro_rules! info {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_INFO {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Info] ");
			print!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		} else {
			print!("x");
		}
	};
}
#[macro_export]
macro_rules! infoln {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_INFO {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Info] ");
			println!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
// pub fn info(s: &str) {
// 	if LOG_LEVEL >= LOGLEVEL_INFO {
// 		println!("[Info] {}", s);
// 	}
// }

#[macro_export]
macro_rules! warn {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_WARN {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Warning] ");
			print!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
#[macro_export]
macro_rules! warnln {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_WARN {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Warning] ");
			println!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
// pub fn warn(s: &str) {
// 	if LOG_LEVEL >= LOGLEVEL_WARN {
// 		println!("[Warning] {}", s);
// 	}
// }

#[macro_export]
macro_rules! error {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_ERROR {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Error] ");
			print!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
#[macro_export]
macro_rules! errorln {
	($($arg:tt)*) => {
		if crate::mi::logger::get_level() >= crate::mi::logger::LOGLEVEL_ERROR {
			let guard =  crate::mi::logger::_MUTEX.lock();
			print!("[Error] ");
			println!($($arg)*);
			crate::mi::logger::flush();
			drop(guard);
		}
	};
}
// pub fn error(s: &str) {
// 	if LOG_LEVEL >= LOGLEVEL_ERROR {
// 		eprintln!("[Error] {}", s);
// 	}
// }
