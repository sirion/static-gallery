
pub trait Replace {
	fn replace(&self, pattern: &[u8], replacement: &[u8]) -> Self;
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
}
