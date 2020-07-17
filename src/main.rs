/* TODO :
- translations ?
- waiting window ?
- support for multiple paths like `/home/user/{folder1,folder2}/src/`
- fix case_insensitive for file filter regex
- set call limits on backend ?
- create a buffer file in each browsing folder ? (faster next loading, followed by async update)
- listen file changes in browsing folders
- finishing implementing arguments options in cli_parsing
- or light/dark theme ? or both ?
- display move folders as big button with folder icon ? maybe custom icon ?
- send and use token with front-end ?
- display icon if file already in working folder
- sort move list by child files count ?
- can remove keyword with `-keyword` in move search
- allow separators for multiple search for move
- about popup
*/

#![allow(clippy::needless_return)]

const WORKING_FOLDER_ENV_NAME: &str = "FRACTUS-ATLAS_WORKING_FOLDER";
const CONFIGURATION_ENV_NAME: &str = "FRACTUS-ATLAS_CONFIG";

use rand::Rng;
use std::sync::{Arc, Mutex};

mod cli_parsing;
mod configuration;
mod logger;
mod user_data;
mod webserver;
mod window;

use logger::LogLevel;

fn main() {
	human_panic::setup_panic!();

	let (instructions, temp_logs) = cli_parsing::CliInstructions::new();
	let logger = crate::logger::new(&instructions);
	for (attributes, content) in temp_logs {
		charlie_buffalo::push(&logger, attributes, Some(&content));
	}
	let configuration = configuration::Configuration::from((&instructions, logger.clone()));

	let mut rng = rand::thread_rng();

	let mut user_data = user_data::UserData::default();
	user_data.internal_server_port = rng.gen_range(1024, 65535);
	user_data.browsing_folders = instructions.browsing_folders.clone();
	user_data.debug = instructions.debug;

	let arc_user_data: Arc<Mutex<user_data::UserData>> = Arc::new(Mutex::new(user_data));

	webserver::run(
		instructions.clone(),
		std::sync::Arc::clone(&arc_user_data),
		logger.clone(),
	);

	window::run(
		instructions,
		configuration,
		std::sync::Arc::clone(&arc_user_data),
		logger,
	);
}

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
