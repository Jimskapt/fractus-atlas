// TODO : translations ?
// TODO : better error handling (search for `unwrap`)

#![allow(clippy::needless_return)]

use rand::seq::IteratorRandom;
use rand::Rng;
use std::sync::{Arc, Mutex};

mod instructions;

fn main() {
	let working_folder_env = format!(
		"{}_WORKING_FOLDER",
		String::from(env!("CARGO_PKG_NAME")).to_uppercase()
	);
	let mut default_working_folder_name = std::path::PathBuf::new();
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
	let default_filter_regex = "\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(jfif))$";

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
				.default_value(&default_filter_regex),
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
				.default_value(""),
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
	let configuration = std::path::PathBuf::from(matches.value_of("config_file_path").unwrap());
	let configuration: Configuration = if configuration.exists() {
		let configuration = std::fs::read_to_string(configuration);

		match configuration {
			Ok(configuration) => match toml::from_str(&configuration) {
				Ok(configuration) => configuration,
				Err(e) => {
					println!(
							"INFO: error while parsing configuration, falling back to default configuration (this is not fatal) : {}",
							e
						);

					Configuration::default()
				}
			},
			Err(e) => {
				println!(
					"INFO: error while reading configuration, falling back to default configuration (this is not fatal) : {}",
					e
				);

				Configuration::default()
			}
		}
	} else {
		if show_debug {
			println!(
				"DEBUG: configuration file does not exists at {}, creating it with default value",
				&configuration.as_path().to_str().unwrap()
			);
		}

		let result = Configuration::default();

		if let Some(folder) = &configuration.parent() {
			match std::fs::create_dir_all(folder) {
				Ok(_) => match std::fs::write(&configuration, toml::to_vec(&result).unwrap()) {
					Ok(_) => {}
					Err(e) => {
						println!(
							"INFO: can not create file {:?} (this is not fatal) : {}",
							&configuration, e
						);
					}
				},
				Err(e) => {
					println!(
						"INFO: can not create folder {:?} (this is not fatal) : {}",
						&folder, e
					);
				}
			}
		}

		result
	};

	if show_debug {
		println!("DEBUG: debug mode activated");
		println!();
		println!("DEBUG: root targets are {:?}", targets);
		println!("DEBUG: filter regex is {:?}", filter);
		println!("DEBUG: working folder is {:?}", working_folder);
		println!("DEBUG: sorting files by {:?}", sort_order);
		println!("DEBUG: {:?}", &configuration);
		println!();
	}

	if !working_folder.exists() {
		println!(
			"DEBUG: working folder {:?} does not exists, attempting to create it",
			working_folder
		);
		println!();

		std::fs::create_dir_all(&working_folder).unwrap_or_else(|_| {
			panic!(
				"INFO: can not creating working folder : {:?}",
				&working_folder
			)
		});
	}

	let working_folder = dunce::canonicalize(working_folder).unwrap();

	// TODO : fix case_insensitive
	let file_regex = match regex::RegexBuilder::new(&filter)
		.case_insensitive(true)
		.build()
	{
		Ok(res) => res,
		Err(e) => {
			println!(
					"INFO: compilation of filter regex {} has failed, falling back to default ({}) : {}",
					&filter, &default_filter_regex, e
				);

			regex::RegexBuilder::new(&default_filter_regex)
				.case_insensitive(true)
				.build()
				.unwrap()
		}
	};

	let window_title = format!("{} V{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

	let mut rng = rand::thread_rng();
	let selected_port = rng.gen_range(1024, 65535);

	let user_data = UserData {
		internal_server_port: selected_port,
		position: 0,
		images: vec![],
		targets: targets.into_iter().map(String::from).collect(),
		show_debug,
		token: String::new(),
	};

	let arc_udata: Arc<Mutex<UserData>> = Arc::new(Mutex::new(user_data));

	let html = include_str!("main.html")
		.replace(
			r#"<script src="main.js"></script>"#,
			&format!("<script>{}</script>", include_str!("main.js")),
		)
		.replace(
			r#"<link rel="stylesheet" href="main.css">"#,
			&format!(
				"<style type=\"text/css\">{}</style>",
				include_str!("main.css").replace(
					"background: white; /* configuration.background */",
					&format!("background: {};", &configuration.background)
				)
			),
		);

	let arc_for_window_data = std::sync::Arc::clone(&arc_udata);
	let main_window = web_view::builder()
		.title(&window_title)
		.content(web_view::Content::Html(&html))
		.size(800, 600)
		.resizable(true)
		.debug(show_debug)
		.user_data(arc_for_window_data)
		.invoke_handler(|webview, arg| {
			if show_debug {
				println!("DEBUG: receiving {}", &arg);
			}

			match serde_json::from_str(&arg).unwrap() {
				instructions::Instruction::Previous => {
					instructions::Previous(webview, show_debug);
				}
				instructions::Instruction::Next => {
					instructions::Next(webview, show_debug);
				}
				instructions::Instruction::Random => {
					instructions::Random(webview, show_debug);
				}
				instructions::Instruction::SetPosition { value } => {
					instructions::SetPosition(webview, show_debug, value);
				}
				instructions::Instruction::Move { into } => {
					instructions::Move(webview, show_debug, &working_folder, into);
				}
				instructions::Instruction::ShowBrowseTarget { id } => {
					instructions::ShowBrowseTarget(webview, show_debug, id);
				}
				instructions::Instruction::BrowseTargetFolders {
					folders,
					toggle_window,
				} => {
					instructions::BrowseTargetFolders(
						webview,
						show_debug,
						&file_regex,
						String::from(sort_order),
						folders,
						toggle_window,
					);
				}
			}

			return Ok(());
		})
		.build()
		.unwrap_or_else(|e| panic!("Can not build main window : {}", e));

	let arc_for_ws = std::sync::Arc::clone(&arc_udata);
	std::thread::spawn(move || {
		let port = { arc_for_ws.lock().unwrap().internal_server_port };

		if show_debug {
			println!(
				"DEBUG: internal web server will be launched at http://127.0.0.1:{}/",
				&port
			);
		}

		iron::Iron::new(move |req: &mut iron::prelude::Request| {
			let current = &arc_for_ws.lock().unwrap().get_current();

			let path_requested = format!("{}", req.url);
			let mut token = format!("http://127.0.0.1:{}/", port);
			token += &arc_for_ws.lock().unwrap().token;

			if path_requested == token {
				if show_debug {
					println!("DEBUG: receiving request to {}", &path_requested);
				}

				let file = std::fs::read(&current);

				match file {
					Ok(path) => {
						let mime = tree_magic::from_u8(&path);
						let mut res = iron::Response::with((iron::status::Ok, path));
						res.headers
							.set_raw("Content-Type", vec![mime.as_bytes().to_vec()]);
						Ok(res)
					}
					Err(e) => {
						if show_debug {
							eprintln!("ERROR: can not get file {} because : {}", current, e);
						}

						match e.kind() {
							std::io::ErrorKind::NotFound => Err(iron::IronError::new(
								StringError(String::from("404 : NOT FOUND")),
								iron::status::Status::NotFound,
							)),
							std::io::ErrorKind::PermissionDenied => Err(iron::IronError::new(
								StringError(String::from("401 : UNAUTHORIZED")),
								iron::status::Status::Unauthorized,
							)),
							_ => Err(iron::IronError::new(
								StringError(String::from("500 : INTERNAL ERROR")),
								iron::status::Status::InternalServerError,
							)),
						}
					}
				}
			} else {
				if show_debug {
					println!(
						"DEBUG: the token does not match with request {}",
						&path_requested
					);
				}

				Err(iron::IronError::new(
					StringError(String::from("403 : FORBIDDEN")),
					iron::status::Status::Forbidden,
				))
			}
		})
		.http(&format!("127.0.0.1:{}", &port))
		.unwrap();
	});

	let mut folders: Vec<String> = vec![];
	for entry in std::fs::read_dir(&working_folder).unwrap() {
		let path = entry.unwrap().path();
		if path.is_dir() {
			folders.push(String::from(path.file_name().unwrap().to_str().unwrap()));
		}
	}

	let arc_for_dispatch = std::sync::Arc::clone(&arc_udata);

	main_window
		.handle()
		.dispatch(move |main_window| {
			let targets = { arc_for_dispatch.lock().unwrap().targets.clone() };
			let internal_server_port = { arc_for_dispatch.lock().unwrap().internal_server_port };

			// ****** TARGETS ******

			let mut targets_buffer = String::from("['");
			targets_buffer += &targets
				.into_iter()
				.map(|target| target.replace("\\", "\\\\").replace("\'", "\\'"))
				.collect::<Vec<String>>()
				.join("','");
			targets_buffer += "']";

			if targets_buffer == "['']" {
				targets_buffer = String::from("[]");
			}

			// ****** FOLDERS ******

			let mut folders_buffer = String::from("['");
			folders_buffer += &folders.join("','");
			folders_buffer += "']";

			if folders_buffer == "['']" {
				folders_buffer = String::from("[]");
			}

			// ****** sending ******

			main_window.eval(&format!(
				"STANDALONE_MODE=false;
				App.data.debug = {};
				App.data.internal_server_port = {};
				App.remote.receive.set_targets({});
				App.methods.do_open(false);
				App.remote.receive.set_folders({});",
				if show_debug { "true" } else { "false" },
				internal_server_port,
				&targets_buffer,
				&folders_buffer,
			))
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

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone)]
struct Image {
	current: std::path::PathBuf,
}
pub struct UserData {
	internal_server_port: usize,
	position: usize,
	images: Vec<Image>,
	targets: Vec<String>,
	show_debug: bool,
	token: String,
}
impl UserData {
	fn get_current(&self) -> String {
		if !self.images.is_empty() {
			return String::from(
				self.images[self.position]
					.current
					.as_path()
					.to_str()
					.unwrap(),
			);
		} else {
			return String::from("");
		}
	}

	fn set_position(&mut self, value: usize) {
		let mut set = value;
		if !self.images.is_empty() {
			if value > (self.images.len() - 1) {
				set = 0;
			}
		} else {
			set = 0;
		}

		let mut token = String::new();
		let mut rng_limit = rand::thread_rng();
		for _ in 1..rng_limit.gen_range(32, 64) {
			let mut rng_item = rand::thread_rng();
			token.push(ALPHABET.chars().choose(&mut rng_item).unwrap());
		}

		if self.show_debug {
			println!("DEBUG: new token is {}", token);
		}

		self.position = set;
		self.token = token;
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

	fn get_next(&self) -> String {
		let pos = if self.position >= self.images.len() - 1 {
			0
		} else {
			self.position + 1
		};

		return String::from(self.images[pos].current.as_path().to_str().unwrap());
	}

	fn get_previous(&self) -> String {
		let pos = if self.position == 0 {
			self.images.len() - 1
		} else {
			self.position - 1
		};

		return String::from(self.images[pos].current.as_path().to_str().unwrap());
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

#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
struct Configuration {
	background: String,
}
impl Default for Configuration {
	fn default() -> Configuration {
		Configuration {
			background: String::from("#FFFFFF"),
		}
	}
}

#[derive(Debug)]
struct StringError(String);
impl std::fmt::Display for StringError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}
impl std::error::Error for StringError {
	fn description(&self) -> &str {
		&*self.0
	}
}
