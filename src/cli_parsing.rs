type TemporaryLog = (Vec<(String, String)>, String);

#[derive(Clone)]
pub struct CliInstructions {
	pub debug: bool,
	pub browsing_folders: Option<Vec<String>>,
	pub sort: String,
	pub filter: String,
	pub working_folder: String,
	pub configuration_path: String,
}
impl Default for CliInstructions {
	fn default() -> Self {
		let working_folder =
			std::path::PathBuf::from(String::from(env!("CARGO_PKG_NAME")).to_lowercase());

		let mut configuration_path = working_folder.clone();
		configuration_path.push("conf.toml");

		let working_folder = String::from(working_folder.as_path().to_str().unwrap_or_default());
		let configuration_path =
			String::from(configuration_path.as_path().to_str().unwrap_or_default());

		/*
		let default_exclude = format!(
			"/^(.*(\\.git).*)|({}(/|\\)?.*)|(\\..+)$/i",
			default_working_folder_name.replace(".", "\\.")
		);
		*/

		CliInstructions {
			debug: false,
			browsing_folders: None,
			sort: String::from("modified"),
			filter: String::from("\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(jfif))$"),
			working_folder,
			configuration_path,
		}
	}
}
impl CliInstructions {
	pub fn new() -> (Self, Vec<TemporaryLog>) {
		let mut result = CliInstructions::default();
		let mut logs = vec![];

		let working_folder = result.working_folder;
		let filter = result.filter;
		let sort = result.sort;
		let configuration_path = result.configuration_path;

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
					.help("If set, show maximum debug information")
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
					.help(r#"The browsing folders where search for files, separated by a coma ","."#)
					.required(false),
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
					.default_value(&configuration_path),
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

		if let Some(browsing_folders) = matches.value_of("TARGETS") {
			result.browsing_folders = Some(
				browsing_folders
					.split(',')
					.map(|i| String::from(i.trim()))
					.collect(),
			);
		} else {
			result.browsing_folders = None;
		}

		result.filter = String::from(matches.value_of("filter").unwrap_or(&filter));
		result.working_folder = String::from(
			matches
				.value_of("working_folder")
				.unwrap_or(&working_folder),
		);
		result.sort = String::from(matches.value_of("sort").unwrap_or(&sort));
		result.configuration_path = String::from(
			matches
				.value_of("config_file_path")
				.unwrap_or(&configuration_path),
		);

		let working_folder = std::path::Path::new(&result.working_folder);
		if !working_folder.exists() {
			logs.push((
				vec![crate::LogLevel::INFO.into()],
				format!(
					"working folder {} does not exists, attempting to create it",
					working_folder.display()
				),
			));

			std::fs::create_dir_all(&working_folder).unwrap_or_else(|_| {
				panic!(
					"can not creating working folder : {}",
					working_folder.display()
				);
			});
		}

		result.working_folder = String::from(
			dunce::canonicalize(working_folder)
				.unwrap_or_default()
				.as_path()
				.to_str()
				.unwrap_or_default(),
		);

		logs.push((
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "startup").into(),
			],
			"running up the app".into(),
		));

		if result.debug {
			logs.push((
				vec![
					crate::LogLevel::DEBUG.into(),
					charlie_buffalo::Attr::new("component", "app").into(),
				],
				"debug mode activated".into(),
			));
		}

		logs.push((
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Flag::from("PRIVATE_DATA").into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			format!("browsing folders are {:?}", result.browsing_folders),
		));
		logs.push((
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			format!("filter regex is {}", result.filter),
		));
		logs.push((
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			format!("working folder is {}", result.working_folder),
		));
		logs.push((
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			format!("sorting files by {}", result.sort),
		));

		return (result, logs);
	}
}
