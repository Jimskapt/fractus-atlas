pub fn next(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		local_user_data.next();

		format!(
			"App.remote.receive.set_current({}, '{}', '{}');
			App.remote.receive.preload('{}');
			App.remote.receive.preload('{}');",
			&local_user_data.position,
			&local_user_data
				.get_current()
				.replace("\\", "\\\\")
				.replace("'", "\\'"),
			&local_user_data.token,
			&local_user_data
				.get_next()
				.replace("\\", "\\\\")
				.replace("'", "\\'"),
			&local_user_data
				.get_previous()
				.replace("\\", "\\\\")
				.replace("'", "\\'")
		)
	};

	if show_debug {
		println!("sending `{}` to view from Next()", &js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}
