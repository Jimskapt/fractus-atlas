pub fn show_browse_folder_window(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
	id: usize,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		let browsing_folders = &mut local_user_data.browsing_folders;

		match browsing_folders {
			Some(browsing_folders) => {
				if id < browsing_folders.len() {
					if let Some(path) = tinyfiledialogs::select_folder_dialog(
						"📂 Browse to folder ...",
						&dunce::canonicalize(browsing_folders[id].clone())
							.unwrap_or_default()
							.as_path()
							.to_str()
							.unwrap_or_default(),
					) {
						browsing_folders[id] = path;
					}
				} else if let Some(path) = tinyfiledialogs::select_folder_dialog(
					"📂 Browse to folder ...",
					&dunce::canonicalize(".")
						.unwrap_or_default()
						.as_path()
						.to_str()
						.unwrap_or_default(),
				) {
					browsing_folders.push(path);
				}
			}
			None => {
				if let Some(path) = tinyfiledialogs::select_folder_dialog(
					"📂 Browse to folder ...",
					&dunce::canonicalize(".")
						.unwrap_or_default()
						.as_path()
						.to_str()
						.unwrap_or_default(),
				) {
					*browsing_folders = Some(vec![path]);
				}
			}
		}

		let mut browsing_folders_buffer = String::from("[");
		browsing_folders_buffer += &local_user_data
			.browsing_folders
			.clone()
			.unwrap()
			.into_iter()
			.map(|browsing_folder| format!("{}", web_view::escape(&browsing_folder)))
			.collect::<Vec<String>>()
			.join(",");
		browsing_folders_buffer += "]";

		format!(
			"App.remote.receive.set_browsing_folders({});",
			&browsing_folders_buffer
		)
	};

	crate::window::run_js(webview, &js_instruction, logger).unwrap();
}
