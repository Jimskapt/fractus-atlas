// TODO : translations ?
// TODO : better error handling (search for `unwrap`)

use rand::seq::IteratorRandom;
use rand::Rng;
use serde_derive::*;

fn main() {
	let working_folder_env = format!(
		"{}_WORKING_FOLDER",
		String::from(env!("CARGO_PKG_NAME")).to_uppercase()
	);
	let mut default_working_folder_name = std::path::PathBuf::new();
	default_working_folder_name.push(".");
	default_working_folder_name.push(String::from(env!("CARGO_PKG_NAME")).to_lowercase());
	let config_env = format!(
		"{}_CONFIG",
		String::from(env!("CARGO_PKG_NAME")).to_uppercase()
	);
	let default_config_file_name = "conf.toml";
	let mut default_config_file_path = default_working_folder_name.clone();
	default_config_file_path.push(default_config_file_name);
	let default_config_file_path = default_config_file_path.as_path().to_str().unwrap();
	/*
	let default_exclude = format!(
		"/^(.*(\\.git).*)|({}(/|\\)?.*)|(\\..+)$/i",
		default_working_folder_name.replace(".", "\\.")
	);
	*/

	let app = clap::App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(
			clap::Arg::with_name("working_folder")
				.short("w")
				.long("folder")
				.value_name("FOLDER_PATH")
				.help("Sets the working folder path for this app (mainly where images will be copied).")
				.takes_value(true)
				.required(false)
				.env(&working_folder_env)
				.default_value(&default_working_folder_name.as_path().to_str().unwrap()),
		)
		.arg(
			clap::Arg::with_name("debug")
				.short("d")
				.long("debug")
				.help("If set, show maximum debug informations")
				.takes_value(false)
				.required(false),
		)
		.arg(
			clap::Arg::with_name("filter")
				.short("f")
				.long("filter")
				.value_name("REGEX")
				.help("Exclude paths which match with this Regular Expression. The match is case insensitive, to the moment.")
				.takes_value(true)
				.required(false)
				.default_value("\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(jfif))$"),
		)
		.arg(
			clap::Arg::with_name("sort")
				.short("s")
				.long("sort")
				.takes_value(true)
				.possible_values(&["name", "modified"])
				.value_name("order")
				.help("Sort images by name or by edit date")
				.required(false)
				.default_value("modified"),
		)
		.arg(
			clap::Arg::with_name("TARGETS")
				.help(r#"The folders where search for files, separated by a coma ","."#)
				.required(false)
				.default_value("."),
		)
		.arg(
			clap::Arg::with_name("config_file_path")
				.short("c")
				.long("config")
				.value_name("FILE_PATH")
				.help("Sets the TOML configuration file path for this application")
				.takes_value(true)
				.required(false)
				.env(&config_env)
				.default_value(&default_config_file_path),
		);
	/*
	.arg(
		clap::Arg::with_name("move-mode")
			.short("m")
			.long("mode")
			.takes_value(true)
			.possible_values(&["move", "copy"])
			.help("Sets if the `move` instruction will move the file or copy it")
			.required(false)
			.default_value("move"),
	)
	.arg(
		clap::Arg::with_name("recursive")
			.short("r")
			.long("recursive")
			.help("If set, the application search other files recursively in sub-folders")
			.takes_value(false)
			.required(false),
	)
	.arg(
		clap::Arg::with_name("exclude")
			.short("x")
			.long("exclude")
			.value_name("REGEX")
			.help("The filter for local file name, which is a Regular Expression")
			.takes_value(true)
			.required(false)
			.default_value(&default_exclude),
	)
	*/

	let matches = app.get_matches();

	let show_debug = matches.is_present("debug");
	// TODO : support for multiple paths like `/home/user/{folder1,folder2}/src/`
	let targets: Vec<&str> = matches
		.value_of("TARGETS")
		.unwrap()
		.split(',')
		.map(|i| i.trim())
		.collect();
	let filter = matches.value_of("filter").unwrap();
	let working_folder = matches.value_of("working_folder").unwrap();
	let sort_order = matches.value_of("sort").unwrap();
	let working_folder = std::path::Path::new(working_folder);
	let settings = std::path::PathBuf::from(matches.value_of("config_file_path").unwrap());
	let settings: Settings = if settings.exists() {
		let settings = std::fs::read_to_string(settings).unwrap();

		toml::from_str(&settings).unwrap()
	} else {
		if show_debug {
			println!(
				"DEBUG: settings file does not exists at {}, creating it with default value",
				&settings.as_path().to_str().unwrap()
			);
		}

		if let Some(folder) = &settings.parent() {
			std::fs::create_dir_all(folder).unwrap();
		}

		let result = Settings::default();

		std::fs::write(settings, toml::to_vec(&result).unwrap()).unwrap();

		result
	};

	if show_debug {
		println!("DEBUG: debug mode activated");
		println!();
		println!("DEBUG: root targets are {:?}", targets);
		println!("DEBUG: filter regex is {:?}", filter);
		println!("DEBUG: working folder is {:?}", working_folder);
		println!("DEBUG: sorting files by {:?}", sort_order);
		println!("DEBUG: settings are {:?}", &settings);
		println!();
	}

	if !working_folder.exists() {
		println!(
			"DEBUG: working folder {:?} does not exists, attempting to create it",
			working_folder
		);
		println!();

		std::fs::create_dir_all(working_folder).unwrap();
	}

	let working_folder = dunce::canonicalize(working_folder).unwrap();

	if show_debug {
		println!("DEBUG: starting searching in root targets");
	}

	let file_regex = regex::RegexBuilder::new(filter)
		.case_insensitive(true)
		.build()
		.unwrap();

	let mut images: Vec<Image> = vec![];
	for root in targets {
		let mut temp: Vec<Image> = std::fs::read_dir(root)
			.unwrap()
			.map(|i| {
				let path = dunce::canonicalize(i.unwrap().path()).unwrap();

				Image { current: path }
			})
			.filter(|i| {
				if i.current.is_file() {
					if let Some(name) = i.current.file_name() {
						match name.to_str() {
							Some(file_name) => {
								if file_regex.is_match(file_name) {
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

	if show_debug {
		println!();
		/*
		println!("DEBUG: images = {:?}", images);
		println!();
		*/
		println!("DEBUG: end of searching in root targets");
		println!();
		println!("DEBUG: building web_view window");
	}

	let window_title = format!("{} V{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

	let user_data = UserData {
		position: 0,
		images,
	};

	let html = include_str!("main.html")
		.replace(
			r#"<script src="./main.js"></script>"#,
			&format!("<script>{}</script>", include_str!("main.js")),
		)
		.replace(
			r#"<link rel="stylesheet" href="./main.css">"#,
			&format!(
				"<style type=\"text/css\">{}</style>",
				include_str!("main.css").replace("{{settings.background}}", &settings.background)
			),
		);

	let main_window = web_view::builder()
		.title(&window_title)
		.content(web_view::Content::Html(&html))
		.size(800, 600)
		.resizable(true)
		.debug(show_debug)
		.user_data(user_data)
		.invoke_handler(|webview, arg| {
			match serde_json::from_str(arg).unwrap() {
				Instruction::Previous => {
					webview.user_data_mut().previous();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} to view from previous()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				Instruction::Next => {
					webview.user_data_mut().next();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} to view from next()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				Instruction::Random => {
					webview.user_data_mut().random();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} to view from random()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				Instruction::SetPosition { value } => {
					webview.user_data_mut().set_position(value);

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} to view from random()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				Instruction::Move { into } => {
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
						let alphabet =
							"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
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

					let temp = new_path
						.clone()
						.as_path()
						.to_str()
						.unwrap()
						.replace("\\", "\\\\");

					udata.images[udata.position].current = new_path;

					webview
						.eval(&format!("new_current(\"{}\");", temp))
						.unwrap();
					webview.eval("next();").unwrap();
					webview.eval("toggle_move_window();").unwrap();

					// TODO : following is duplicate :
					let mut folders: Vec<String> = vec![];
					for entry in std::fs::read_dir(&working_folder).unwrap() {
						let path = entry.unwrap().path();
						if path.is_dir() {
							folders.push(String::from(path.file_name().unwrap().to_str().unwrap()));
						}
					}
					let mut folders_buffer = String::from("[\"");
					folders_buffer += &folders.join("\",\"");
					folders_buffer += "\"]";

					if folders_buffer == "[\"\"]" {
						folders_buffer = String::from("[]");
					}

					webview
						.eval(&format!("set_folders({});", &folders_buffer))
						.unwrap();
				}
				Instruction::Receiving { data } => {
					if show_debug {
						println!("receiving {} from view", data);
					}
				}
			}

			Ok(())
		})
		.build()
		.unwrap();

	let mut images_buffer = String::from("[\"");

	let temp = main_window.user_data().images.clone();
	images_buffer += &temp
		.iter()
		.map(|i| String::from(i.current.to_str().unwrap()))
		.collect::<Vec<String>>()
		.join("\",\"");
	images_buffer += "\"]";

	images_buffer = images_buffer.replace("\\", "\\\\");

	if images_buffer == "[\"\"]" {
		images_buffer = String::from("[]");
	}

	let mut folders: Vec<String> = vec![];
	for entry in std::fs::read_dir(&working_folder).unwrap() {
		let path = entry.unwrap().path();
		if path.is_dir() {
			folders.push(String::from(path.file_name().unwrap().to_str().unwrap()));
		}
	}
	let mut folders_buffer = String::from("[\"");
	folders_buffer += &folders.join("\",\"");
	folders_buffer += "\"]";

	if folders_buffer == "[\"\"]" {
		folders_buffer = String::from("[]");
	}

	if show_debug {
		// println!("DEBUG: sending images to web_view window : {}", images_buffer);
		println!("DEBUG: sending images to web_view window");
	}

	let handle = main_window.handle();
	handle
		.dispatch(move |main_window| {
			main_window
				.eval(&format!("set_images({});", &images_buffer))
				.unwrap();
			main_window
				.eval(&format!("set_folders({});", &folders_buffer))
				.unwrap();

			Ok(())
		})
		.unwrap();

	if show_debug {
		println!("DEBUG: running web_view window");
	}

	main_window.run().unwrap();

	if show_debug {
		println!("DEBUG: end of program");
	}
}

#[derive(Clone)]
struct Image {
	current: std::path::PathBuf,
}
struct UserData {
	position: usize,
	images: Vec<Image>,
}
impl UserData {
	fn set_position(&mut self, value: usize) {
		let mut set = value;
		if !self.images.is_empty() {
			if value > (self.images.len() - 1) {
				set = 0;
			}
		} else {
			set = 0;
		}

		self.position = set;
	}
	fn previous(&mut self) {
		if self.position < 1 {
			if !self.images.is_empty() {
				self.set_position(self.images.len() - 1);
			} else {
				self.set_position(0);
			}
		} else {
			self.set_position(self.position - 1);
		}
	}
	fn next(&mut self) {
		self.set_position(self.position + 1);
	}
	fn random(&mut self) {
		if !self.images.is_empty() {
			let mut rng = rand::thread_rng();
			self.set_position(rng.gen_range(0, self.images.len() - 1));
		} else {
			self.set_position(0);
		}
	}
}

#[derive(Deserialize)]
#[serde(tag = "instruction", rename_all = "PascalCase")]
enum Instruction {
	Previous,
	Next,
	Random,
	SetPosition { value: usize },
	Move { into: String },
	Receiving { data: String },
}

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
struct Settings {
	background: String,
}
impl Default for Settings {
	fn default() -> Settings {
		Settings {
			background: String::from("#FFFFFF"),
		}
	}
}
