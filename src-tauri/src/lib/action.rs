use std::sync::Arc;
use tokio::sync::RwLock;

use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub enum AppAction {
	ChangePosition(isize),
	ChangeRandomPosition,
	Move(usize),
	RestoreImage,
}

#[async_recursion::async_recursion]
pub async fn apply_action(
	state: Arc<RwLock<crate::AppState>>,
	action: &AppAction,
) -> Result<bool, ()> {
	let mut changed = false;

	match action {
		AppAction::ChangePosition(step) => {
			// TODO : check .try_into().unwrap() here

			let old_position = state.read().await.current_position;
			let max_val: isize = state.read().await.images.len().try_into().unwrap();

			if let Some(position) = old_position {
				let position_int: isize = position.try_into().unwrap();
				let mut new_position: isize = position_int + step;

				if new_position >= max_val {
					let diff = new_position - max_val;

					new_position = 0 + diff;
				}

				if new_position < 0 {
					new_position += max_val;
				}

				let new_position_usize: usize = new_position.try_into().unwrap();
				state.write().await.set_position(Some(new_position_usize));

				changed = (old_position != state.read().await.current_position);
			}
		}
		AppAction::ChangeRandomPosition => {
			let max_val = state.read().await.images.len();

			let new_position = rand::thread_rng().gen_range(0..max_val);

			state.write().await.set_position(Some(new_position));

			changed = true;
		}
		AppAction::Move(id) => {
			let current_position = state.read().await.current_position;
			if let Some(position) = current_position {
				let mut new_path = std::path::PathBuf::new();

				{
					let rstate = state.read().await;
					let output_folder = rstate.settings.output_folders.get(*id);

					if let Some(output) = output_folder {
						if let Some(image) = state.read().await.images.get(position) {
							let old_path = image.get_current();

							let output_path = super::build_absolute_path(
								&output.path,
								rstate.settings_path.clone(),
							);

							std::fs::create_dir_all(&output_path).unwrap();

							new_path = output_path.join::<String>(
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
						let mut state_w = state.write().await;

						if let Some(image) = state_w.images.get_mut(position) {
							image.moved = Some(new_path);
							state_w.set_position(Some(position));
							state_w.settings.steps_after_move
						} else {
							todo!()
						}
					};

					apply_action(state, &AppAction::ChangePosition(steps))
						.await
						.unwrap();
				}
			}
		}
		AppAction::RestoreImage => {
			let current_position = state.read().await.current_position;
			if let Some(position) = current_position {
				{
					if let Some(image) = state.read().await.images.get(position) {
						let current_path = image.get_current();
						let mut origin_path = image.origin.clone();

						while origin_path.exists() {
							let rand_id: String = rand::thread_rng()
								.sample_iter(&Alphanumeric)
								.take(8)
								.map(char::from)
								.collect();

							let origin_parent = origin_path.parent().unwrap();

							origin_path = origin_parent.join(format!(
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

						if let Some(parent) = origin_path.parent() {
							std::fs::create_dir_all(parent).ok();
						}

						if current_path != origin_path
							&& std::fs::copy(&current_path, &origin_path).is_ok()
						{
							if trash::delete(&current_path).is_ok() {
								changed = true;
							} else {
								trash::delete(&origin_path).ok();
								// TODO : warn user
							}
						}
					}
				}

				if changed {
					let mut state_w = state.write().await;
					if let Some(image) = state_w.images.get_mut(position) {
						image.moved = None;
						state_w.set_position(Some(position));
					}
				}
			}
		}
	}

	Ok(changed)
}
