use std::sync::{Arc, RwLock};

use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub enum AppAction {
	ChangePosition(isize),
	Move(usize),
	RestoreImage,
}

pub fn apply_action(state: Arc<RwLock<crate::AppState>>, action: &AppAction) -> bool {
	let mut changed = false;

	match action {
		AppAction::ChangePosition(step) => {
			// TODO : check .try_into().unwrap() here

			let old_position = state.read().unwrap().current_position;
			let max_val: isize = state.read().unwrap().images.len().try_into().unwrap();

			if let Some(position) = old_position {
				let position: isize = position.try_into().unwrap();
				let mut new_position: isize = position + step;

				if new_position >= max_val {
					let diff = new_position - max_val;

					new_position = 0 + diff;
				}

				if new_position < 0 {
					new_position = max_val + new_position;
				}

				let new_position = new_position.try_into().unwrap();
				state.write().unwrap().current_position = Some(new_position);
				state.write().unwrap().display_path = format!(
					"{}",
					state
						.read()
						.unwrap()
						.images
						.get(new_position)
						.unwrap()
						.get_current()
						.display()
				);

				changed = (old_position != state.read().unwrap().current_position);
			}
		}
		AppAction::Move(id) => {
			let current_position = state.read().unwrap().current_position;
			if let Some(position) = current_position {
				let mut new_path = std::path::PathBuf::new();

				{
					let rstate = state.read().unwrap();
					let output_folder = rstate.settings.output_folders.get(*id);

					if let Some(output) = output_folder {
						if let Some(image) = state.read().unwrap().images.get(position) {
							let old_path = image.get_current();
							new_path = output.path.join::<String>(
								image
									.origin
									.file_name()
									.map(std::ffi::OsStr::to_string_lossy)
									.unwrap_or_else(|| {
										let rand_id: String = rand::thread_rng()
											.sample_iter(&Alphanumeric)
											.take(8)
											.map(char::from)
											.collect();

										format!(
											"{}{}",
											rand_id,
											image
												.origin
												.extension()
												.map(|val| format!(".{}", val.to_string_lossy()))
												.unwrap_or(String::new())
										)
										.into()
									})
									.into(),
							);

							if let Ok(path) = super::exec_move(&old_path, &new_path) {
								changed = true;
								new_path = path;
							}
						}
					}
				}

				if changed {
					let steps = {
						let mut state_w = state.write().unwrap();

						if let Some(image) = state_w.images.get_mut(position) {
							image.moved = Some(new_path);
							state_w.display_path = format!("{}", image.get_current().display());
							state_w.settings.steps_after_move
						} else {
							todo!()
						}
					};

					apply_action(state, &AppAction::ChangePosition(steps));
				}
			}
		}
		AppAction::RestoreImage => {
			let current_position = state.read().unwrap().current_position;
			if let Some(position) = current_position {
				let mut new_path = std::path::PathBuf::new();

				{
					if let Some(image) = state.read().unwrap().images.get(position) {
						let old_path = image.get_current();
						new_path = image.origin.clone();

						while new_path.exists() {
							let rand_id: String = rand::thread_rng()
								.sample_iter(&Alphanumeric)
								.take(8)
								.map(char::from)
								.collect();

							let origin_parent = new_path.parent().unwrap();

							new_path = origin_parent.join(format!(
								"{}{}{}",
								image
									.origin
									.file_stem()
									.map(|val| format!("{}-d-", val.to_string_lossy()))
									.unwrap_or(String::new()),
								rand_id,
								image
									.origin
									.extension()
									.map(|val| format!(".{}", val.to_string_lossy()))
									.unwrap_or(String::new())
							));
						}

						if let Some(parent) = new_path.parent() {
							std::fs::create_dir_all(parent).ok();
						}

						if old_path != new_path && std::fs::copy(&old_path, &new_path).is_ok() {
							if trash::delete(&old_path).is_ok() {
								changed = true;
							} else {
								trash::delete(&new_path).ok();
								// TODO : warn user
							}
						}
					}
				}

				if changed {
					let mut state_w = state.write().unwrap();
					if let Some(image) = state_w.images.get_mut(position) {
						image.moved = Some(new_path);
						state_w.display_path = format!("{}", image.get_current().display())
					}
				}
			}
		}
	}

	changed
}
