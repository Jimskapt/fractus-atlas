pub fn random(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
) {
	let js_instruction = {
		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		local_user_data.random();

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
			"DEBUG: sending\n```js\n{}\n```\nto view from random()\n",
			&js_instruction
		);
	}
	webview.eval(&js_instruction).unwrap();
}
