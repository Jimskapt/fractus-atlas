pub fn browse_target_folders(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	file_regex: &regex::Regex,
	sort_order: String,
	folders: Vec<String>,
	toggle_window: bool,
) {
	let js_instruction = {
		if show_debug {
			println!("DEBUG: starting searching in targets");
		}

		let mut images: Vec<crate::user_data::Image> = vec![];
		for root in &folders {
			let mut temp: Vec<crate::user_data::Image> = std::fs::read_dir(root)
				.unwrap()
				.map(|i| {
					let path = dunce::canonicalize(i.unwrap().path()).unwrap();

					crate::user_data::Image { current: path }
				})
				.filter(|i| {
					if i.current.is_file() {
						if let Some(name) = i.current.file_name() {
							match name.to_str() {
								Some(file_name) => {
									if file_regex.is_match(file_name) {
										if show_debug {
											println!("DEBUG: adding file {:?}", file_name);
										}

										return true;
									} else {
										if show_debug {
											println!(
												"DEBUG: file {:?} does not match file filter regex",
												file_name
											);
										}
										return false;
									}
								}
								None => {
									if show_debug {
										println!(
											"DEBUG: can not get UTF-8 file name of {:?}",
											name
										);
									}
									return false;
								}
							}
						} else {
							if show_debug {
								println!("DEBUG: can not get file name of {:?}", i.current);
							}
							return false;
						}
					} else {
						if show_debug {
							println!("DEBUG: {:?} is not a file", i.current);
						}
						return false;
					}
				})
				.collect();

			images.append(&mut temp);
		}

		if show_debug {
			println!();
			println!("DEBUG: end of searching in root targets");
			println!();
			println!("DEBUG: sorting found files by order : {}", &sort_order);
		}

		if sort_order == "modified" {
			images.sort_by(|a, b| {
				let b_time = b
					.current
					.metadata()
					.unwrap()
					.modified()
					.unwrap_or_else(|_| std::time::SystemTime::now());
				let a_time = a
					.current
					.metadata()
					.unwrap()
					.modified()
					.unwrap_or_else(|_| std::time::SystemTime::now());

				return b_time.cmp(&a_time);
			});
		}

		if show_debug {
			println!();
			println!("DEBUG: end of sorting found files");
		}

		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		local_user_data.images = images;
		local_user_data.set_position(0);

		format!(
			"App.remote.receive.set_images_count({});
			App.remote.receive.set_current({}, '{}', '{}');",
			&local_user_data.images.len(),
			&local_user_data.position,
			&local_user_data
				.get_current()
				.replace("\\", "\\\\")
				.replace("'", "\\'"),
			&local_user_data.token
		)
	};

	if show_debug {
		println!(
			"sending `{}` to view from BrowseTargetFolders()",
			&js_instruction
		);
	}
	webview.eval(&js_instruction).unwrap();

	if toggle_window {
		let js_instruction = "App.methods.toggle_open_window();";

		if show_debug {
			println!(
				"sending `{}` to view from BrowseTargetFolders()",
				&js_instruction
			);
		}
		webview.eval(&js_instruction).unwrap();
	}
}
