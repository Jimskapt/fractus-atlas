#[derive(Clone)]
pub struct CliInstructions {
	pub debug: bool,
	pub targets: Vec<String>,
	pub sort: String,
	pub filter: String,
	pub working_folder: String,
	pub configuration_path: String,
}
impl Default for CliInstructions {
	fn default() -> Self {
		let mut working_folder = std::path::PathBuf::new();
		working_folder.push(String::from(env!("CARGO_PKG_NAME")).to_lowercase());

		let mut configuration_path = working_folder.clone();
		configuration_path.push("conf.toml");

		let working_folder = String::from(working_folder.as_path().to_str().unwrap());
		let configuration_path = String::from(configuration_path.as_path().to_str().unwrap());

		/*
		let default_exclude = format!(
			"/^(.*(\\.git).*)|({}(/|\\)?.*)|(\\..+)$/i",
			default_working_folder_name.replace(".", "\\.")
		);
		*/

		CliInstructions {
			debug: false,
			targets: vec![String::new()],
			sort: String::from("modified"),
			filter: String::from("\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(jfif))$"),
			working_folder,
			configuration_path,
		}
	}
}
impl CliInstructions {
	pub fn new() -> Self {
		let mut result = CliInstructions::default();

		let default_targets = result.targets.join(",");

		let working_folder = result.working_folder;
		let filter = result.filter;
		let sort = result.sort;

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
					.env(crate::WORKING_FOLDER_ENV_NAME)
					.default_value(&working_folder),
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
					.default_value(&filter),
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
					.default_value(&sort),
			)
			.arg(
				clap::Arg::with_name("TARGETS")
					.help(r#"The folders where search for files, separated by a coma ","."#)
					.required(false)
					.default_value(&default_targets),
			)
			.arg(
				clap::Arg::with_name("config_file_path")
					.short("c")
					.long("config")
					.value_name("FILE_PATH")
					.help("Sets the TOML configuration file path for this application")
					.takes_value(true)
					.required(false)
					.env(crate::CONFIGURATION_ENV_NAME)
					.default_value(&result.configuration_path),
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

		result.debug = matches.is_present("debug");

		// TODO : support for multiple paths like `/home/user/{folder1,folder2}/src/`
		result.targets = matches
			.value_of("TARGETS")
			.unwrap()
			.split(',')
			.map(|i| String::from(i.trim()))
			.collect();

		result.filter = String::from(matches.value_of("filter").unwrap());
		result.working_folder = String::from(matches.value_of("working_folder").unwrap());
		result.sort = String::from(matches.value_of("sort").unwrap());
		result.configuration_path = String::from(matches.value_of("config_file_path").unwrap());

		if result.debug {
			println!("DEBUG: debug mode activated");
			println!();
			println!("DEBUG: root targets are {:?}", result.targets);
			println!("DEBUG: filter regex is {:?}", result.filter);
			println!("DEBUG: working folder is {:?}", result.working_folder);
			println!("DEBUG: sorting files by {:?}", result.sort);
			println!();
		}

		let working_folder = std::path::PathBuf::from(result.working_folder);
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

		result.working_folder = String::from(
			dunce::canonicalize(working_folder)
				.unwrap()
				.as_path()
				.to_str()
				.unwrap(),
		);

		return result;
	}
}
