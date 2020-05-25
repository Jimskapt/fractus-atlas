use rand::seq::IteratorRandom;
use rand::Rng;

#[derive(Clone)]
pub struct Image {
	pub current: std::path::PathBuf,
}
pub struct UserData {
	pub internal_server_port: usize,
	pub position: usize,
	pub images: Vec<Image>,
	pub targets: Vec<String>,
	pub debug: bool,
	pub token: String,
}
impl UserData {
	pub fn get_current(&self) -> String {
		if !self.images.is_empty() {
			return String::from(
				self.images[self.position]
					.current
					.as_path()
					.to_str()
					.unwrap(),
			);
		} else {
			return String::from("");
		}
	}

	pub fn set_position(&mut self, value: usize) {
		let mut set = value;
		if !self.images.is_empty() {
			if value > (self.images.len() - 1) {
				set = 0;
			}
		} else {
			set = 0;
		}

		let mut token = String::new();
		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32, 64) {
			let mut rng_item = rand::thread_rng();
			token.push(crate::ALPHABET.chars().choose(&mut rng_item).unwrap());
		}

		if self.debug {
			println!("DEBUG: new token is {}", token);
		}

		self.position = set;
		self.token = token;
	}

	pub fn previous(&mut self) {
		if self.position < 1 {
			if !self.images.is_empty() {
				self.set_position(self.images.len() - 1);
			} else {
				self.set_position(0);
			}
		} else {
			self.set_position(self.position - 1);
		}
	}

	pub fn next(&mut self) {
		self.set_position(self.position + 1);
	}

	pub fn get_next(&self) -> String {
		let pos = if self.position >= self.images.len() - 1 {
			0
		} else {
			self.position + 1
		};

		return String::from(self.images[pos].current.as_path().to_str().unwrap());
	}

	pub fn get_previous(&self) -> String {
		let pos = if self.position == 0 {
			self.images.len() - 1
		} else {
			self.position - 1
		};

		return String::from(self.images[pos].current.as_path().to_str().unwrap());
	}

	pub fn random(&mut self) {
		if !self.images.is_empty() {
			let mut rng = rand::thread_rng();
			self.set_position(rng.gen_range(0, self.images.len() - 1));
		} else {
			self.set_position(0);
		}
	}
}
