pub fn open_current_file(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
) {
	let active_file = webview.user_data_mut().lock().unwrap().get_active();

	if let Some(file) = active_file {
		let path = format!("{}", file.current.display());

		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
				charlie_buffalo::Attr::new("component", "webview").into(),
				charlie_buffalo::Attr::new("event", "open_current_file").into(),
			],
			Some(&format!("open file {}", path)),
		);

		opener::open(path).ok();
	}
}

pub fn open_current_folder(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	logger: charlie_buffalo::ConcurrentLogger,
) {
	let active_file = webview.user_data_mut().lock().unwrap().get_active();

	if let Some(file) = active_file {
		let path = format!("{}", file.current.parent().unwrap().display());

		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
				charlie_buffalo::Attr::new("component", "webview").into(),
				charlie_buffalo::Attr::new("event", "open_current_folder").into(),
			],
			Some(&format!("open folder {}", path)),
		);

		opener::open(path).ok();
	}
}
