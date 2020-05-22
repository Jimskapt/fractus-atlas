// TODO : translations ?
// TODO : better error handling (search for `unwrap`)

#![allow(clippy::needless_return)]

use rand::Rng;

mod instructions;

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

	let file_regex = regex::RegexBuilder::new(filter)
		.case_insensitive(true)
		.build()
		.unwrap();

	let window_title = format!("{} V{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

	let user_data = UserData {
		position: 0,
		images: vec![],
		targets: targets.into_iter().map(String::from).collect(),
	};

	let html = include_str!("main.html")
		.replace(
			r#"<script src="./main.js"></script>"#,
			&format!(
				"<script>{}</script>",
				include_str!("main.js").replace(
					"debug: false, // context.debug",
					&format!("debug: {},", if show_debug { "true" } else { "false" })
				)
			),
		)
		.replace(
			r#"<link rel="stylesheet" href="./main.css">"#,
			&format!(
				"<style type=\"text/css\">{}</style>",
				include_str!("main.css").replace(
					"background: white; /* settings.background */",
					&format!("background: {};", &settings.background)
				)
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
		.unwrap();

	let mut folders: Vec<String> = vec![];
	for entry in std::fs::read_dir(&working_folder).unwrap() {
		let path = entry.unwrap().path();
		if path.is_dir() {
			folders.push(String::from(path.file_name().unwrap().to_str().unwrap()));
		}
	}

	let handle = main_window.handle();
	handle
		.dispatch(move |main_window| {
			// ****** TARGETS ******

			let mut targets_buffer = String::from("['");
			targets_buffer += &main_window
				.user_data()
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

			main_window
				.eval(&format!(
					"App.remote.receive.set_targets({});",
					&targets_buffer
				))
				.unwrap();

			main_window.eval("App.methods.do_open(false);").unwrap();

			// ****** FOLDERS ******

			let mut folders_buffer = String::from("['");
			folders_buffer += &folders.join("','");
			folders_buffer += "']";

			if folders_buffer == "['']" {
				folders_buffer = String::from("[]");
			}

			main_window
				.eval(&format!(
					"App.remote.receive.set_folders({});",
					&folders_buffer
				))
				.unwrap();

			// ****** IMAGES COUNT ******

			main_window
				.eval(&format!(
					"App.remote.receive.set_images_count({});",
					&main_window.user_data().images.len()
				))
				.unwrap();

			// ****** CURRENT IMAGE ******

			if !main_window.user_data().images.is_empty() {
				main_window
					.eval(&format!(
						"App.remote.receive.set_current({}, '{}');",
						&main_window.user_data().position,
						&main_window
							.user_data()
							.get_current()
							.replace("\\", "\\\\")
							.replace("'", "\\'")
					))
					.unwrap();
			}

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
pub struct UserData {
	position: usize,
	images: Vec<Image>,
	targets: Vec<String>,
}
impl UserData {
	fn get_current(&self) -> String {
		return String::from(
			self.images[self.position]
				.current
				.as_path()
				.to_str()
				.unwrap(),
		);
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
