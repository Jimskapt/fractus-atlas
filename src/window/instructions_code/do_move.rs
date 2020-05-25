use rand::seq::IteratorRandom;
use rand::Rng;

pub fn do_move(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	show_debug: bool,
	working_folder: String,
	into: String,
) {
	let working_folder = std::path::PathBuf::from(&working_folder);

	{
		let mut local_user_data = webview.user_data_mut().lock().unwrap();
		let image = local_user_data
			.images
			.get(local_user_data.position)
			.unwrap();

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
			for _ in 1..rng_limit.gen_range(6, 12) {
				let mut rng_item = rand::thread_rng();
				new_name.push(crate::ALPHABET.chars().choose(&mut rng_item).unwrap());
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

		let pos = local_user_data.position;
		local_user_data.images[pos].current = new_path;
	}

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

	let js_instruction = format!(
		"App.remote.send('Next');
		App.methods.toggle_move_window();
		App.remote.receive.set_folders({});",
		&folders_buffer
	);

	if show_debug {
		println!("sending `{}` to view from Move()", &js_instruction);
	}
	webview.eval(&js_instruction).unwrap();
}
