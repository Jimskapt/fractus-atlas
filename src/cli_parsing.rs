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
			targets: vec![String::from(".")],
			sort: String::from("modified"),
			filter: String::from("\\.((png)|(tiff)|(tif)|(bmp)|(jpg)|(jpeg)|(gif)|(jfif))$"),
			working_folder,
			configuration_path,
		}
	}
}
impl CliInstructions {
	pub fn new() -> (Self, charlie_buffalo::ConcurrentLogger) {
		let mut result = CliInstructions::default();

		let default_targets = result.targets.join(",");

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

		result.targets = matches
			.value_of("TARGETS")
			.unwrap_or_default()
			.split(',')
			.map(|i| String::from(i.trim()))
			.collect();

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
			if result.debug {
				println!(
					"INFO: working folder {:?} does not exists, attempting to create it",
					working_folder
				);
				println!();
			}

			std::fs::create_dir_all(&working_folder).unwrap_or_else(|_| {
				panic!(
					"INFO: can not creating working folder : {:?}",
					&working_folder
				)
			});
		}

		result.working_folder = String::from(
			dunce::canonicalize(working_folder)
				.unwrap_or_default()
				.as_path()
				.to_str()
				.unwrap_or_default(),
		);

		let mut logpath = std::path::PathBuf::from(&result.working_folder);
		logpath.push("logs.msgpack");
		let logpath_for_dispatch = logpath.clone();
		let logpath_for_drop = logpath;
		let show_logs = result.debug;

		let logger = charlie_buffalo::concurrent_logger_from(charlie_buffalo::Logger::new(
			charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
				let mut new_log = log;

				let attributes: Vec<(String, String)> =
					vec![charlie_buffalo::Attr::new(
						"time",
						format!("{}", chrono::offset::Local::now()),
					)
					.into()];
				for attribute in attributes {
					new_log.attributes.insert(attribute.0, attribute.1);
				}

				if show_logs
					|| new_log.attributes.get("level").unwrap_or(
						&charlie_buffalo::ValueAsString::as_string(&crate::LogLevel::INFO),
					) > &charlie_buffalo::ValueAsString::as_string(&crate::LogLevel::DEBUG)
				{
					println!("{}", new_log);
				}

				let mut result: Vec<charlie_buffalo::Log> = rmp_serde::decode::from_slice(
					std::fs::read(&logpath_for_dispatch)
						.unwrap_or_default()
						.as_slice(),
				)
				.unwrap_or_default();
				result.push(new_log);
				std::fs::write(
					&logpath_for_dispatch,
					rmp_serde::encode::to_vec(&result).unwrap(),
				)
				.unwrap();
			})),
			charlie_buffalo::new_dropper(Box::from(move |logger: &charlie_buffalo::Logger| {
				(*logger).push(
					vec![
						crate::LogLevel::DEBUG.into(),
						charlie_buffalo::Attr::new("stage", "stop").into(),
					],
					Some("stopping the app"),
				);

				println!(
					"\n(logs should be inside file {})\n",
					&logpath_for_drop.as_path().to_str().unwrap()
				);
			})),
		));

		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("stage", "startup").into(),
			],
			Some("running up the app"),
		);

		if result.debug {
			charlie_buffalo::push(
				&logger,
				vec![crate::LogLevel::DEBUG.into()],
				Some("debug mode activated"),
			);
		}

		charlie_buffalo::push(
			&logger,
			vec![crate::LogLevel::DEBUG.into()],
			Some(&format!("root targets are {:?}", result.targets)),
		);
		charlie_buffalo::push(
			&logger,
			vec![crate::LogLevel::DEBUG.into()],
			Some(&format!("filter regex is {:?}", result.filter)),
		);
		charlie_buffalo::push(
			&logger,
			vec![crate::LogLevel::DEBUG.into()],
			Some(&format!("working folder is {:?}", result.working_folder)),
		);
		charlie_buffalo::push(
			&logger,
			vec![crate::LogLevel::DEBUG.into()],
			Some(&format!("sorting files by {:?}", result.sort)),
		);

		return (result, logger);
	}
}
