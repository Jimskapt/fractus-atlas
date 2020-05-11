// TODO : translations ?

use rand::Rng;

fn main() {
	/*
	let config_env = format!(
		"{}_CONFIG",
		String::from(env!("CARGO_PKG_NAME")).to_uppercase()
	);
	let default_working_folder_name = String::from(env!("CARGO_PKG_NAME")).to_lowercase();
	let default_config_file_name = "conf.toml";
	let default_config_file_path = format!(
		"./{}/{}",
		default_working_folder_name, default_config_file_name
	);
	let default_exclude = format!(
		"/^(.*(\\.git).*)|({}(/|\\)?.*)|(\\..+)$/i",
		default_working_folder_name.replace(".", "\\.")
	);
	*/

	let app = clap::App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		/*
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
		)
		.arg(
			clap::Arg::with_name("recursive")
				.short("r")
				.long("recursive")
				.help("If set, the application search other files recursively in sub-folders")
				.takes_value(false)
				.required(false),
		)
		*/
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
				.default_value("\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(webp))$"),
		)
		/*
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
		.arg(
			clap::Arg::with_name("TARGETS")
				.help(r#"The folders where search for files, separated by a coma ","."#)
				.required(false)
				.default_value("."),
		);

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

	if show_debug {
		println!("DEBUG: debug mode activated");
		println!();
		println!("DEBUG: root targets are {:?}", targets);
		println!("DEBUG: filter regex is {:?}", filter);
		println!();
		println!("DEBUG: starting searching in root targets");
	}

	let file_regex = regex::RegexBuilder::new(filter)
		.case_insensitive(true)
		.build()
		.unwrap();

	let mut images: Vec<std::path::PathBuf> = vec![];
	for root in targets {
		let mut temp: Vec<std::path::PathBuf> = std::fs::read_dir(root)
			.unwrap()
			.map(|i| i.unwrap().path())
			.filter(|i| {
				if i.as_path().is_file() {
					if let Some(name) = i.file_name() {
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
							println!("DEBUG: can not get file name of {:?}", i);
						}
						return false;
					}
				} else {
					if show_debug {
						println!("DEBUG: {:?} is not a file", i);
					}
					return false;
				}
			})
			.collect();

		images.append(&mut temp);
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

	struct UserData {
		position: usize,
		images: Vec<std::path::PathBuf>,
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
	let user_data = UserData {
		position: 0,
		images,
	};

	let main_window = web_view::builder()
		.title(&window_title)
		.content(web_view::Content::Html(include_str!("viewer.html")))
		.size(800, 600)
		.resizable(true)
		.debug(show_debug)
		.user_data(user_data)
		.invoke_handler(|webview, arg| {
			match arg {
				"previous" => {
					webview.user_data_mut().previous();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} from previous()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				"next" => {
					webview.user_data_mut().next();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} from next()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				"random" => {
					webview.user_data_mut().random();

					let js_instruction =
						format!("set_position({});", &webview.user_data().position);
					if show_debug {
						println!("sending {} from random()", js_instruction);
					}
					webview.eval(&js_instruction).unwrap();
				}
				"move" => {
					println!("move instruction received");
				}
				_ => unimplemented!(),
			}

			Ok(())
		})
		.build()
		.unwrap();

	let mut images_buffer = String::from("[\"");

	let temp = &main_window.user_data().images;
	images_buffer += &temp
		.iter()
		.map(|i| String::from(dunce::canonicalize(i).unwrap().as_path().to_str().unwrap()))
		.collect::<Vec<String>>()
		.join("\",\"");
	images_buffer += "\"]";

	images_buffer = images_buffer.replace("\\", "\\\\");

	if images_buffer == "[\"\"]" {
		images_buffer = String::from("[]");
	}

	if show_debug {
		// println!("DEBUG: sending images to web_view window : {}", images_buffer);
		println!("DEBUG: sending images to web_view window");
	}

	let handle = main_window.handle();
	handle
		.dispatch(move |main_window| main_window.eval(&format!("set_images({});", &images_buffer)))
		.unwrap();

	if show_debug {
		println!("DEBUG: running web_view window");
	}

	main_window.run().unwrap();

	if show_debug {
		println!("DEBUG: end of program");
	}
}
