fn main() {
	/*
	let config_env = format!(
		"{}_CONFIG",
		String::from(env!("CARGO_PKG_NAME")).to_uppercase()
	);
	let default_config_file_name = format!(
		".{}.conf.toml",
		String::from(env!("CARGO_PKG_NAME")).to_lowercase()
	);
	let default_config_file_path = format!("./{}", default_config_file_name);
	let default_exclude = format!(
		"/^(.*(\\.git).*)|({})|(\\..+)$/i",
		default_config_file_name.replace(".", "\\.")
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
		.unwrap(); // .expect("ERROR: the filter is not regex-valid : {}", e);

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
		println!("DEBUG: images = {:?}", images);
		println!();
		println!("DEBUG: end of searching in root targets");
		println!();
		println!("DEBUG: building web_view window");
	}

	let window_title = format!("{} V{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

	let mut wv = web_view::builder()
		.title(&window_title)
		.content(web_view::Content::Html(include_str!("viewer.html")))
		.size(800, 600)
		.resizable(true)
		.debug(true)
		.user_data(images)
		.invoke_handler(|_webview, arg| {
			match arg {
				"move" => unimplemented!(),
				_ => unimplemented!(),
			}

			Ok(())
		})
		.build()
		.unwrap();

	let mut images_buffer = String::from("[\"");
	images_buffer += &wv
		.user_data()
		.into_iter()
		.map(|i| i.as_path().to_str().unwrap())
		.collect::<Vec<&str>>()
		.join("\",\"");
	images_buffer += "\"]";

	images_buffer = images_buffer.replace("\\", "\\\\");

	if show_debug {
		println!("DEBUG: sending images to web_view window");
	}

	wv.eval(&format!("set_images({});", &images_buffer))
		.unwrap();

	if show_debug {
		println!("DEBUG: running web_view window");
	}

	wv.run().unwrap();

	if show_debug {
		println!("DEBUG: end of program");
	}
}
