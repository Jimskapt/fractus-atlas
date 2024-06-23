use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub enum AppAction {
	ChangePosition(isize),
	ChangeRandomPosition,
	Move(usize),
	RestoreImage,
}

#[async_recursion::async_recursion]
pub async fn apply_action(state: &crate::AppState, action: &AppAction) -> Result<bool, ()> {
	let mut changed = false;

	log::debug!("*** NEW action = {action:?}");

	match action {
		AppAction::ChangePosition(step) => {
			// TODO : check .try_into().unwrap() here

			let old_position = state.images.read().await.get_current_pos();
			log::debug!("old_position = {old_position:?}");

			let max_val: isize = state.images.read().await.len().try_into().unwrap();
			log::debug!("max_val = {max_val:?}");

			if let Some(position) = old_position {
				let position_int: isize = position.try_into().unwrap();
				log::debug!("position_int = {position_int}");

				let mut new_position: isize = position_int + step;

				if new_position >= max_val {
					let diff = new_position - max_val;

					new_position = 0 + diff;
				}

				if new_position < 0 {
					new_position += max_val;
				}

				log::debug!("new_position = {new_position}");

				let new_position_usize: usize = new_position.try_into().unwrap();
				{
					log::trace!("trying write MLK-123");

					state
						.images
						.write()
						.await
						.set_position(Some(new_position_usize), &mut *state.refresh.write().await);

					log::trace!("finished write MLK-123");
				}

				changed = (old_position != state.images.read().await.get_current_pos());

				log::debug!("changed = {changed}");
			} else {
				log::debug!("no current image");
			}
		}
		AppAction::ChangeRandomPosition => {
			let max_val = state.images.read().await.len();

			let new_position = rand::thread_rng().gen_range(0..max_val);

			{
				log::trace!("trying write SWR-564");
				state
					.images
					.write()
					.await
					.set_position(Some(new_position), &mut *state.refresh.write().await);
				log::trace!("finished write SWR-564");
			}

			changed = true;
		}
		AppAction::Move(id) => {
			let current_pos = state.images.read().await.get_current_pos();
			if let Some(position) = current_pos {
				let mut new_path = std::path::PathBuf::new();

				{
					let settings = state.settings.read().await;
					let output_folder = settings.output_folders.get(*id);

					if let Some(output) = output_folder {
						if let Some(image) = state.images.read().await.get_pos(position) {
							let output_path = common::build_absolute_path(
								&output.path,
								state.settings_path.read().await.clone(),
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

							log::debug!("new_path = {new_path:?}");

							changed = true;
						}
					}
				}

				if changed {
					log::trace!("trying write SEV-853");
					state
						.images
						.write()
						.await
						.move_current(new_path, &mut *state.refresh.write().await)
						.unwrap();
					log::trace!("finished write SEV-853");
					let steps = state.settings.read().await.steps_after_move;

					apply_action(state, &AppAction::ChangePosition(steps))
						.await
						.unwrap();
				}
			} else {
				log::debug!("no current image");
			}
		}
		AppAction::RestoreImage => {
			log::trace!("trying write VZN-841");
			if let Err(err) = state
				.images
				.write()
				.await
				.restore_current(&mut *state.refresh.write().await)
			{
				log::error!("error while restaure : {err}");
			}
			log::trace!("finished write VZN-841");

			changed = true;
		}
	}

	log::debug!("changed = {changed}");

	Ok(changed)
}
