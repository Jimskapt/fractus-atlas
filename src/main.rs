/* TODO :
- translations ?
- can move back after move
- open current file in external
- fix manual target path field change
- add delete button on target fields
- waiting window ?
- support for multiple paths like `/home/user/{folder1,folder2}/src/`
- fix case_insensitive for file filter regex
- notification center
- fix call limits on backend
*/

#![allow(clippy::needless_return)]

const WORKING_FOLDER_ENV_NAME: &str = "FRACTUS-ATLAS_WORKING_FOLDER";
const CONFIGURATION_ENV_NAME: &str = "FRACTUS-ATLAS_CONFIG";

use rand::Rng;
use std::sync::{Arc, Mutex};

mod cli_parsing;
mod configuration;
mod user_data;
mod webserver;
mod window;

fn main() {
	human_panic::setup_panic!();

	let instructions = cli_parsing::CliInstructions::new();
	let configuration = configuration::Configuration::from(&instructions);

	let mut rng = rand::thread_rng();

	let mut user_data = user_data::UserData::default();
	user_data.internal_server_port = rng.gen_range(1024, 65535);
	user_data.targets = instructions.targets.clone();
	user_data.debug = instructions.debug;

	let arc_user_data: Arc<Mutex<user_data::UserData>> = Arc::new(Mutex::new(user_data));

	webserver::run(instructions.clone(), std::sync::Arc::clone(&arc_user_data));
	window::run(
		instructions.clone(),
		configuration,
		std::sync::Arc::clone(&arc_user_data),
	);

	if instructions.debug {
		println!("DEBUG: end of program");
	}
}

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
