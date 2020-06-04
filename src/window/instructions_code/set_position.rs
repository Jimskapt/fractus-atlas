pub fn set_position(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
	value: usize,
) {
	let js_instructions: String = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();

		local_user_data.set_position(value);

		let mut result = String::new();
		result += &local_user_data.get_js_set_active();
		result += &local_user_data.get_js_preloads();

		result
	};

	crate::window::run_js(webview, &js_instructions, logger).unwrap();
}
