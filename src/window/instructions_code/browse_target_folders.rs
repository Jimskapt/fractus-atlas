pub fn browse_target_folders(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	file_regex: &regex::Regex,
	sort_order: String,
	folders: Vec<String>,
	toggle_window: bool,
) {
	let mut js_instructions = {
		if show_debug {
			println!("DEBUG: starting searching in targets");
		}

		let mut images: Vec<crate::user_data::Image> = vec![];
		for root in &folders {
			match std::fs::read_dir(root) {
				Ok(folder) => {
					images.append(
						&mut folder
							.map(|i| {
								crate::user_data::Image::from(
									dunce::canonicalize(i.unwrap().path()).unwrap_or_default(),
								)
							})
							.filter(|i| {
								if i.current.is_file() {
									if let Some(name) = i.current.file_name() {
										match name.to_str() {
											Some(file_name) => {
												if file_regex.is_match(file_name) {
													if show_debug {
														println!(
															"DEBUG: adding file {:?}",
															file_name
														);
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
											println!(
												"DEBUG: can not get file name of {:?}",
												i.current
											);
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
							.collect::<Vec<crate::user_data::Image>>(),
					);
				}
				Err(e) => {
					eprintln!("ERROR: can not read folder {} : {}", &root, e);
				}
			}
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
			println!("DEBUG: end of sorting found files");
		}

		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		local_user_data.images = images;
		local_user_data.set_position(0);

		let mut result = format!(
			"App.remote.receive.set_images_count({});\n",
			&local_user_data.images.len()
		);

		result += &local_user_data.get_js_set_active();

		result
	};

	if toggle_window {
		js_instructions += "App.methods.toggle_open_window();";
	}

	crate::window::run_js(webview, &js_instructions, show_debug).unwrap();
}
