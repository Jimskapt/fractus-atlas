pub fn show_browse_target(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	id: usize,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();

		if id < local_user_data.targets.len() {
			if let Some(path) = tinyfiledialogs::select_folder_dialog(
				"📂 Browse to folder ...",
				&dunce::canonicalize(local_user_data.targets[id].clone())
					.unwrap()
					.as_path()
					.to_str()
					.unwrap(),
			) {
				local_user_data.targets[id] = path;
			}
		} else if let Some(path) = tinyfiledialogs::select_folder_dialog(
			"📂 Browse to folder ...",
			&dunce::canonicalize(".")
				.unwrap()
				.as_path()
				.to_str()
				.unwrap(),
		) {
			local_user_data.targets.push(path);
		}

		let mut targets_buffer = String::from("['");
		targets_buffer += &local_user_data
			.targets
			.clone()
			.into_iter()
			.map(|target| target.replace("\\", "\\\\").replace("\'", "\\'"))
			.collect::<Vec<String>>()
			.join("','");
		targets_buffer += "']";

		if targets_buffer == "['']" {
			targets_buffer = String::from("[]");
		}

		format!("App.remote.receive.set_targets({});", &targets_buffer)
	};

	if show_debug {
		println!(
			"sending `{}` to view from ShowBrowseTarget()",
			&js_instruction
		);
	}
	webview.eval(&js_instruction).unwrap();
}
