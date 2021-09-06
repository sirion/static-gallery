use crate::errorln;

pub fn exit_on_error<T>(res: Result<T, String>) -> T {
	match res {
		Ok(t) => t,
		Err(e) => {
			errorln!("{}", e);
			std::process::exit(1);
		}
	}
}

