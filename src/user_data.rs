use rand::seq::IteratorRandom;
use rand::Rng;

#[derive(Default)]
pub struct UserData {
	pub internal_server_port: usize,
	pub position: Option<usize>,
	pub images: Vec<Image>,
	pub browsing_folders: Option<Vec<String>>,
	pub debug: bool,
}
impl UserData {
	pub fn get_active(&self) -> Option<Image> {
		if !self.images.is_empty() {
			if let Some(position) = self.position {
				if position < self.images.len() {
					return Some(self.images[position].clone());
				}
			}
		}

		return None;
	}

	fn compute_previous_pos(&self, position: usize) -> usize {
		if position < 1 {
			if !self.images.is_empty() {
				self.images.len() - 1
			} else {
				0
			}
		} else {
			position - 1
		}
	}

	pub fn go_to_previous(&mut self) {
		if let Some(position) = self.position {
			self.position = Some(self.compute_previous_pos(position));
		} else {
			self.position = None;
		}
	}

	pub fn get_previous(&self) -> Option<Image> {
		if !self.images.is_empty() {
			if let Some(position) = self.position {
				return Some(self.images[self.compute_previous_pos(position)].clone());
			}
		}

		return None;
	}

	fn compute_next_pos(&self, position: usize) -> usize {
		if position >= self.images.len() - 1 {
			0
		} else {
			position + 1
		}
	}

	pub fn go_to_next(&mut self) {
		if let Some(position) = self.position {
			self.position = Some(self.compute_next_pos(position));
		} else {
			self.position = None;
		}
	}

	pub fn get_next(&self) -> Option<Image> {
		if !self.images.is_empty() {
			if let Some(position) = self.position {
				return Some(self.images[self.compute_next_pos(position)].clone());
			}
		}

		return None;
	}

	pub fn go_to_random(&mut self) {
		if !self.images.is_empty() {
			let mut rng = rand::thread_rng();
			self.position = Some(rng.gen_range(0..self.images.len()));
		} else {
			self.position = None;
		}
	}

	pub fn set_position(&mut self, position: usize) {
		if !self.images.is_empty() {
			if position < self.images.len() {
				// && position >= 0
				self.position = Some(position);
			} else {
				self.position = Some(self.images.len() - 1);
			}
		} else {
			self.position = None;
		}
	}

	pub fn get_js_preloads(&self) -> String {
		let mut result = String::new();

		if let Some(previous) = self.get_previous() {
			result += &format!(
				"App.remote.receive.preload({});\n",
				web_view::escape(&previous.token)
			);
		}
		if let Some(next) = self.get_next() {
			result += &format!(
				"App.remote.receive.preload({});\n",
				web_view::escape(&next.token)
			);
		}

		return result;
	}

	pub fn get_js_set_active(&self) -> String {
		let mut result = String::new();

		if let Some(active) = self.get_active() {
			result += &format!(
				"App.remote.receive.set_active({}, {}, {}, {});\n",
				self.position.unwrap(),
				web_view::escape(active.current.as_path().to_str().unwrap()),
				web_view::escape(&active.token),
				active.origin != active.current
			);
		}

		return result;
	}
}

#[derive(Clone)]
pub struct Image {
	pub origin: std::path::PathBuf,
	pub current: std::path::PathBuf,
	pub token: String,
}
impl std::convert::From<std::path::PathBuf> for Image {
	fn from(from: std::path::PathBuf) -> Self {
		let mut token = String::new();
		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32..64) {
			let mut rng_item = rand::thread_rng();
			token.push(crate::ALPHABET.chars().choose(&mut rng_item).unwrap());
		}

		Image {
			origin: from.clone(),
			current: from,
			token,
		}
	}
}
