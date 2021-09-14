
pub trait Replace {
	/// Returns a new vec with the given pattern replaced by the replacement
	fn replace(&self, pattern: &[u8], replacement: &[u8]) -> Self;

	/// Returns a new vec with the given replacement instead of the part surrounded by the two patterns (leaves the patterns in)
	fn replace_between(&self, pattern_start: &[u8], pattern_end: &[u8], replacement: &[u8]) -> Self;

	/// Finds the first index of the pattern after the given start index
	fn index_of(&self, pattern: &[u8], start: usize) -> Result<usize, usize>;

	/// Returns a new vec containing what is bewteen the given patterns
	fn between(&self, pattern_start: &[u8], pattern_end: &[u8]) -> Self;
}

impl Replace for Vec<u8> {
	fn replace(&self, pattern: &[u8], replacement: &[u8]) -> Vec<u8> {
		if pattern.len() == 0 {
			return self.clone();
		}


		let mut new: Vec<u8> = Vec::with_capacity(self.len() - pattern.len() + replacement.len());

		let mut i = 0;
		while i < self.len() {
			if self[i] == pattern[0] {
				let mut matching = true;
				for n in 1..pattern.len() {
					if self[i + n] != pattern[n] {
						matching = false;
						break;
					}
				}
				if matching {
					let mut r = Vec::from(replacement);
					new.append(&mut r);
					i = i + pattern.len();
				} else {
					new.push(self[i]);
				}

			} else {
				new.push(self[i]);
			}
			i = i + 1;
		}

		new
	}

	fn replace_between(&self, pattern_start: &[u8], pattern_end: &[u8], replacement: &[u8]) -> Vec<u8> {
		if pattern_start.len() == 0 || pattern_end.len() == 0 {
			return self.to_vec();
		}

		let index_start = match self.index_of(pattern_start, 0) {
			Ok(i) => i,
			Err(_) => {
				return self.to_vec();
			},
		};

		let index_end = match self.index_of(pattern_end, index_start + pattern_start.len()) {
			Ok(i) => i,
			Err(_) => {
				return self.to_vec();
			},
		};

		let mut new: Vec<u8> = Vec::with_capacity(self.len() + replacement.len());

		new.extend_from_slice(&self[0..index_start + pattern_start.len()]);
		new.extend_from_slice(replacement);
		new.extend_from_slice(&self[index_end..]);

		new
	}

	fn index_of(&self, pattern: &[u8], start: usize) -> Result<usize, usize> {
		let mut found = false;

		let mut i = start;
		while i < self.len() {
			if self[i] == pattern[0] {
				let mut matching = true;
				for n in 1..pattern.len() {
					if self[i + n] != pattern[n] {
						matching = false;
						break;
					}
				}
				if matching {
					found = true;
					break;
				}
			}
		 	i = i + 1;
		}

		if found {
			Ok(i)
		} else {
			Err(0)
		}
	}


	fn between(&self, pattern_start: &[u8], pattern_end: &[u8]) -> Vec<u8> {
		if pattern_start.len() == 0 || pattern_end.len() == 0 {
			return self.to_vec();
		}

		let index_start = match self.index_of(pattern_start, 0) {
			Ok(i) => i,
			Err(_) => {
				return self.to_vec();
			},
		};

		let index_end = match self.index_of(pattern_end, index_start + pattern_start.len()) {
			Ok(i) => i,
			Err(_) => {
				return self.to_vec();
			},
		};

		Vec::from(&self[index_start + pattern_start.len()..index_end])
	}

}
