use rand::seq::IteratorRandom;
use rand::Rng;

pub fn do_move(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
	working_folder: String,
	into: String,
	toggle_popup: bool,
) {
	let into = if into == "*move_back_to_origin*" {
		into
	} else {
		sanitize_folder_path(&into.trim(), "-")
	};

	if into.trim() != "" {
		let working_folder = std::path::PathBuf::from(&working_folder);
		let mut toasts = String::new();

		{
			let mut local_user_data = webview.user_data_mut().lock().unwrap();
			if let Some(position) = local_user_data.position {
				let image = local_user_data.images.get(position);

				match image {
					Some(image) => {
						let mut new_path = {
							if into == "*move_back_to_origin*" {
								image.origin.clone()
							} else {
								let mut result = working_folder.clone();
								result.push(&into);
								result
									.push(image.current.as_path().file_name().unwrap_or_default());

								result
							}
						};

						if new_path != image.current {
							while new_path.exists() {
								let name_before_renaming = new_path.clone();

								let mut new_name = String::from(
									new_path
										.as_path()
										.file_stem()
										.unwrap_or_default()
										.to_str()
										.unwrap_or_default(),
								);
								new_name += "-fa_";

								let mut rng_limit = rand::thread_rng();
								for _ in 1..rng_limit.gen_range(6..12) {
									let mut rng_item = rand::thread_rng();
									new_name.push(
										crate::ALPHABET.chars().choose(&mut rng_item).unwrap(),
									);
								}

								new_name += ".";
								new_name += new_path
									.as_path()
									.extension()
									.unwrap_or_default()
									.to_str()
									.unwrap_or_default();

								new_path.set_file_name(new_name);

								charlie_buffalo::push(
									&logger,
									vec![
										crate::LogLevel::INFO.into(),
										charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
										charlie_buffalo::Flag::from("RENAMING").into(),
										charlie_buffalo::Attr::new("component", "webview").into(),
										charlie_buffalo::Attr::new("event", "do_move").into(),
									],
									Some(&format!(
										"file {} already exists, renaming it to {}",
										name_before_renaming.display(),
										new_path.display()
									)),
								);

								toasts += "ToastCenter.data.items.push('";
								toasts += &format!("<strong>{}</strong> already exists, renaming it to <strong>{}</strong>",
									&name_before_renaming.file_name().unwrap_or_default().to_string_lossy().replace("'", "\\'"),
									&new_path.file_name().unwrap_or_default().to_string_lossy().replace("'", "\\'")
								);
								toasts += "', 8000, {class: 'toast info'});\n";
							}

							charlie_buffalo::push(
								&logger,
								vec![
									crate::LogLevel::DEBUG.into(),
									charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
									charlie_buffalo::Attr::new("component", "webview").into(),
									charlie_buffalo::Attr::new("event", "do_move").into(),
								],
								Some(&format!(
									"attempting to move {} in {}",
									image.current.display(),
									new_path.display()
								)),
							);

							if let Some(folder) = new_path.parent() {
								std::fs::create_dir_all(folder).unwrap();
							}

							if std::fs::copy(&image.current, &new_path).is_err() {
								charlie_buffalo::push(
									&logger,
									vec![
										crate::LogLevel::INFO.into(),
										charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
										charlie_buffalo::Attr::new("component", "webview").into(),
										charlie_buffalo::Attr::new("event", "do_move").into(),
									],
									Some(&format!(
										"can not copy file {} to {}",
										image.current.display(),
										new_path.display()
									)),
								);
							} else {
								charlie_buffalo::push(
									&logger,
									vec![
										crate::LogLevel::DEBUG.into(),
										charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
										charlie_buffalo::Attr::new("component", "webview").into(),
										charlie_buffalo::Attr::new("event", "do_move").into(),
									],
									Some(&format!(
										"file {} successfully copied to {}",
										image.current.display(),
										new_path.display()
									)),
								);

								if trash::delete(&image.current).is_err() {
									charlie_buffalo::push(
										&logger,
										vec![
											crate::LogLevel::INFO.into(),
											charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
											charlie_buffalo::Attr::new("component", "webview")
												.into(),
											charlie_buffalo::Attr::new("event", "do_move").into(),
										],
										Some(&format!(
											"can not move file {} to trash (after copied it)",
											image.current.display()
										)),
									);

									if into == "*move_back_to_origin*" {
										toasts += "ToastCenter.data.items.push('";
										toasts += "⚠ file copied back <strong>to its origin</strong> but can not be deleted in (old) moved folder.";
										toasts += "', -1, {classes: 'toast info'});\n";
									} else {
										toasts += "ToastCenter.data.items.push('";
										toasts += &format!("⚠ file copied in <strong>{}</strong> but can not be deleted in its source folder.", into);
										toasts += "', -1, {classes: 'toast info'});\n";
									}
								} else {
									charlie_buffalo::push(
										&logger,
										vec![
											crate::LogLevel::DEBUG.into(),
											charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
											charlie_buffalo::Attr::new("component", "webview")
												.into(),
											charlie_buffalo::Attr::new("event", "do_move").into(),
										],
										Some(&format!(
											"file {} moved to trash (after copied it)",
											image.current.display()
										)),
									);

									if into == "*move_back_to_origin*" {
										toasts += "ToastCenter.data.items.push('";
										toasts +=
											"⏪ <strong>moved back</strong> to origin folder.";
										toasts += "', 3000, {classes: 'toast success'});\n";
									} else {
										toasts += "ToastCenter.data.items.push('";
										toasts += &format!(
											"🚚 moved <i>{}</i> in <strong>{}</strong>.",
											image
												.current
												.file_name()
												.unwrap_or_default()
												.to_string_lossy()
												.replace("'", "\\'"),
											&into
										);
										toasts += "', 3000, {classes: 'toast success'});\n";
									}
								}
							}

							local_user_data.images[position].current = new_path;
						} else {
							charlie_buffalo::push(
								&logger,
								vec![
									crate::LogLevel::INFO.into(),
									charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
									charlie_buffalo::Attr::new("component", "webview").into(),
									charlie_buffalo::Attr::new("event", "do_move").into(),
								],
								Some(&format!(
									"The file is already in {}, so we don't move it",
									image.current.display()
								)),
							);
						}
					}
					None => {
						charlie_buffalo::push(
							&logger,
							vec![
								crate::LogLevel::ERROR.into(),
								charlie_buffalo::Attr::new("component", "webview").into(),
								charlie_buffalo::Attr::new("event", "do_move").into(),
							],
							Some("can not get image information in order to move it"),
						);
					}
				}
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
		folders.sort();

		let mut folders_buffer = String::from("[");
		folders_buffer += &folders
			.iter()
			.map(|folder| format!("{}", web_view::escape(&folder)))
			.collect::<Vec<String>>()
			.join(",");
		folders_buffer += "]";

		let mut js_instructions = format!(
			"App.remote.send('Next');
App.remote.receive.set_move_folders({});
{}",
			&folders_buffer, &toasts
		);

		if toggle_popup {
			js_instructions += "\nApp.methods.toggle_move_window();";
		}

		crate::window::run_js(webview, &js_instructions, logger).unwrap();
	} else {
		crate::window::run_js(webview, "ToastCenter.data.items.push('❌ Please specify a folder name', -1, {class: 'toast error'});", logger).unwrap();
	}
}

const ALLOWED_FOLDER_CHARS: &str =
	"abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn sanitize_folder_path(input: &str, replacement: &str) -> String {
	let mut result = String::new();

	for character in input.chars() {
		if ALLOWED_FOLDER_CHARS.find(character).is_some() {
			result.push(character);
		} else {
			result += replacement;
		}
	}

	return result;
}
