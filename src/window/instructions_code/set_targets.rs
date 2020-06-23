pub fn set_targets(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
	targets: Vec<String>,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();

		local_user_data.targets = targets;

		let mut targets_buffer = String::from("[");
		targets_buffer += &local_user_data
			.targets
			.clone()
			.into_iter()
			.map(|target| format!("{}", web_view::escape(&target)))
			.collect::<Vec<String>>()
			.join(",");
		targets_buffer += "]";

		format!("App.remote.receive.set_targets({});", &targets_buffer)
	};

	crate::window::run_js(webview, &js_instruction, logger).unwrap();
}
