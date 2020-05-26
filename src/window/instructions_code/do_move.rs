use rand::seq::IteratorRandom;
use rand::Rng;

pub fn do_move(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	working_folder: String,
	into: String,
) {
	let working_folder = std::path::PathBuf::from(&working_folder);

	{
		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		let image = local_user_data.images.get(local_user_data.position);

		match image {
			Some(image) => {
				let mut new_path = working_folder.clone();
				new_path.push(&into);
				new_path.push(image.current.as_path().file_name().unwrap_or_default());

				while new_path.exists() {
					new_path = working_folder.clone();
					new_path.push(&into);

					let mut new_name = String::from(
						image
							.current
							.as_path()
							.file_stem()
							.unwrap_or_default()
							.to_str()
							.unwrap_or_default(),
					);
					new_name += "-fa_";

					let mut rng_limit = rand::thread_rng();
					for _ in 1..rng_limit.gen_range(6, 12) {
						let mut rng_item = rand::thread_rng();
						new_name.push(crate::ALPHABET.chars().choose(&mut rng_item).unwrap());
					}

					new_name += ".";
					new_name += image
						.current
						.as_path()
						.extension()
						.unwrap_or_default()
						.to_str()
						.unwrap_or_default();

					new_path.push(new_name);
				}

				if show_debug {
					println!(
						"DEBUG: attempting to move {:?} in {:?}",
						&image.current, &new_path
					);
				}

				if let Some(folder) = new_path.parent() {
					std::fs::create_dir_all(folder).unwrap();
				}

				if std::fs::copy(&image.current, &new_path).is_err() {
					println!(
						"INFO: can not copy file {:?} to {:?}",
						&image.current, &new_path
					);
				} else {
					if show_debug {
						println!(
							"DEBUG: file {:?} successfully copied to {:?}",
							&image.current, &new_path
						);
					}

					if trash::remove(&image.current).is_err() {
						println!(
							"INFO: can not move file {:?} to trash (after copied it)",
							&image.current
						);
					} else if show_debug {
						println!(
							"DEBUG: file {:?} moved to trash (after copied it)",
							&image.current
						);
					}
				}

				let pos = local_user_data.position;
				local_user_data.images[pos].current = new_path;
			}
			None => eprintln!("ERROR: can not get image information in order to move it"),
		}
	}

	// TODO : following is duplicate :
	let mut folders: Vec<String> = vec![];
	if let Ok(childs) = std::fs::read_dir(&working_folder) {
		for entry in childs {
			if let Ok(entry) = entry {
				let path = entry.path();
				if path.is_dir() {
					folders.push(String::from(
						path.file_name()
							.unwrap_or_default()
							.to_str()
							.unwrap_or_default(),
					));
				}
			}
		}
	}
	let mut folders_buffer = String::from("['");
	folders_buffer += &folders.join("','");
	folders_buffer += "']";

	if folders_buffer == "['']" {
		folders_buffer = String::from("[]");
	}

	let js_instruction = format!(
		"App.remote.send('Next');
App.methods.toggle_move_window();
App.remote.receive.set_folders({});",
		&folders_buffer
	);

	if show_debug {
		println!(
			"DEBUG: sending\n```js\n{}\n```\nto view from do_move()\n",
			&js_instruction
		);
	}
	webview.eval(&js_instruction).unwrap();
}
