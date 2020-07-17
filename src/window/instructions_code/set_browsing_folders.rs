pub fn set_browsing_folders(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
	browsing_folders: Vec<String>,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();

		local_user_data.browsing_folders = Some(browsing_folders);

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
