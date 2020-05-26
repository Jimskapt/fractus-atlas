pub fn set_position(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	value: usize,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();

		let new_value = if value > local_user_data.images.len() {
			local_user_data.images.len() - 1
		} else {
			value
		};

		local_user_data.set_position(new_value);

		format!(
			"App.remote.receive.set_current({}, {}, {});
App.remote.receive.preload({});
App.remote.receive.preload({});",
			&local_user_data.position,
			web_view::escape(&local_user_data.get_current()),
			web_view::escape(&local_user_data.token),
			web_view::escape(&local_user_data.get_next()),
			web_view::escape(&local_user_data.get_previous())
		)
	};

	if show_debug {
		println!(
			"DEBUG: sending\n```js\n{}\n```\nto view from set_position()\n",
			&js_instruction
		);
	}
	webview.eval(&js_instruction).unwrap();
}
