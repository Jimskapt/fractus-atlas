use rand::seq::IteratorRandom;
use rand::Rng;
use serde_derive::*;

#[derive(Deserialize)]
#[serde(tag = "instruction", rename_all = "PascalCase")]
pub enum Instruction {
	Previous,
	Next,
	Random,
	SetPosition {
		value: usize,
	},
	Move {
		into: String,
	},
	ShowBrowseTarget {
		id: usize,
	},
	BrowseTargetFolders {
		folders: Vec<String>,
		toggle_window: bool,
	},
}

pub fn Previous(webview: &mut web_view::WebView<crate::UserData>, show_debug: bool) {
	webview.user_data_mut().previous();

	let js_instruction = format!(
		"App.remote.receive.set_current({}, '{}');",
		&webview.user_data().position,
		&webview
			.user_data()
			.get_current()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from previous()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();

	let js_instruction = format!(
		"App.remote.receive.preload('{}');",
		&webview
			.user_data()
			.get_next()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from next()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();

	let js_instruction = format!(
		"App.remote.receive.preload('{}');",
		&webview
			.user_data()
			.get_previous()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from next()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}

pub fn Next(webview: &mut web_view::WebView<crate::UserData>, show_debug: bool) {
	webview.user_data_mut().next();

	let js_instruction = format!(
		"App.remote.receive.set_current({}, '{}');",
		&webview.user_data().position,
		&webview
			.user_data()
			.get_current()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from next()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();

	let js_instruction = format!(
		"App.remote.receive.preload('{}');",
		&webview
			.user_data()
			.get_next()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from next()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();

	let js_instruction = format!(
		"App.remote.receive.preload('{}');",
		&webview
			.user_data()
			.get_previous()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from next()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}

pub fn Random(webview: &mut web_view::WebView<crate::UserData>, show_debug: bool) {
	webview.user_data_mut().random();

	let js_instruction = format!(
		"App.remote.receive.set_current({}, '{}');",
		&webview.user_data().position,
		&webview
			.user_data()
			.get_current()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from random()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}

pub fn SetPosition(
	webview: &mut web_view::WebView<crate::UserData>,
	show_debug: bool,
	value: usize,
) {
	let data = webview.user_data_mut();

	let new_value = if value > data.images.len() {
		data.images.len() - 1
	} else {
		value
	};

	data.set_position(new_value);

	let js_instruction = format!(
		"App.remote.receive.set_current({}, '{}');",
		&webview.user_data().position,
		&webview
			.user_data()
			.get_current()
			.replace("\\", "\\\\")
			.replace("'", "\\'")
	);
	if show_debug {
		println!("sending {} to view from random()", js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}

pub fn Move(
	webview: &mut web_view::WebView<crate::UserData>,
	show_debug: bool,
	working_folder: &std::path::PathBuf,
	into: String,
) {
	let udata = webview.user_data_mut();
	let image = udata.images.get(udata.position).unwrap();

	let mut new_path = working_folder.clone();
	new_path.push(&into);
	new_path.push(image.current.as_path().file_name().unwrap());

	while new_path.exists() {
		new_path = working_folder.clone();
		new_path.push(&into);

		let mut new_name = String::from(
			image
				.current
				.as_path()
				.file_stem()
				.unwrap()
				.to_str()
				.unwrap(),
		);
		new_name += "-fa_";

		let mut rng_limit = rand::thread_rng();
		let alphabet = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
		for _ in 1..rng_limit.gen_range(6, 12) {
			let mut rng_item = rand::thread_rng();
			new_name.push(alphabet.chars().choose(&mut rng_item).unwrap());
		}

		new_name += ".";
		new_name += image
			.current
			.as_path()
			.extension()
			.unwrap()
			.to_str()
			.unwrap();

		new_path.push(new_name);
	}

	if show_debug {
		println!(
			"DEBUG: attempting to move {} in {}",
			&image.current.as_path().to_str().unwrap(),
			&new_path.as_path().to_str().unwrap()
		);
	}

	if let Some(folder) = new_path.parent() {
		std::fs::create_dir_all(folder).unwrap();
	}

	std::fs::copy(&image.current, &new_path).unwrap();
	trash::remove(&image.current).unwrap();

	udata.images[udata.position].current = new_path;

	webview.eval("App.remote.send('Next');").unwrap();
	webview.eval("App.methods.toggle_move_window();").unwrap();

	// TODO : following is duplicate :
	let mut folders: Vec<String> = vec![];
	for entry in std::fs::read_dir(&working_folder).unwrap() {
		let path = entry.unwrap().path();
		if path.is_dir() {
			folders.push(
				String::from(path.file_name().unwrap().to_str().unwrap()).replace("'", "\\'"),
			);
		}
	}
	let mut folders_buffer = String::from("['");
	folders_buffer += &folders.join("','");
	folders_buffer += "']";

	if folders_buffer == "['']" {
		folders_buffer = String::from("[]");
	}

	webview
		.eval(&format!(
			"App.remote.receive.set_folders({});",
			&folders_buffer
		))
		.unwrap();
}

pub fn ShowBrowseTarget(
	webview: &mut web_view::WebView<crate::UserData>,
	_show_debug: bool,
	id: usize,
) {
	let udata = webview.user_data_mut();

	if id < udata.targets.len() {
		if let Some(path) = tinyfiledialogs::select_folder_dialog(
			"📂 Browse to folder ...",
			&dunce::canonicalize(udata.targets[id].clone())
				.unwrap()
				.as_path()
				.to_str()
				.unwrap(),
		) {
			udata.targets[id] = path;
		}
	} else if let Some(path) = tinyfiledialogs::select_folder_dialog(
		"📂 Browse to folder ...",
		&dunce::canonicalize(".")
			.unwrap()
			.as_path()
			.to_str()
			.unwrap(),
	) {
		udata.targets.push(path);
	}

	let mut targets_buffer = String::from("['");
	targets_buffer += &udata
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

	webview
		.eval(&format!(
			"App.remote.receive.set_targets({});",
			&targets_buffer
		))
		.unwrap();
}

pub fn BrowseTargetFolders(
	webview: &mut web_view::WebView<crate::UserData>,
	show_debug: bool,
	file_regex: &regex::Regex,
	sort_order: String,
	folders: Vec<String>,
	toggle_window: bool,
) {
	if show_debug {
		println!("DEBUG: starting searching in root targets");
	}

	let mut images: Vec<crate::Image> = vec![];
	for root in &folders {
		let mut temp: Vec<crate::Image> = std::fs::read_dir(root)
			.unwrap()
			.map(|i| {
				let path = dunce::canonicalize(i.unwrap().path()).unwrap();

				crate::Image { current: path }
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
									println!("DEBUG: can not get UTF-8 file name of {:?}", name);
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

	{
		let udata = webview.user_data_mut();
		udata.images = images;
		udata.set_position(0);
	}

	webview
		.eval(&format!(
			"App.remote.receive.set_images_count({});",
			&webview.user_data().images.len()
		))
		.unwrap();

	webview
		.eval(&format!(
			"App.remote.receive.set_current({}, '{}');",
			&webview.user_data().position,
			&webview
				.user_data()
				.get_current()
				.replace("\\", "\\\\")
				.replace("'", "\\'")
		))
		.unwrap();

	if toggle_window {
		webview.eval("App.methods.toggle_open_window();").unwrap();
	}

	if show_debug {
		println!();
		/*
		println!("DEBUG: images = {:?}", images);
		println!();
		*/
		println!("DEBUG: end of searching in root targets");
	}
}
