// TODO : translations ?
// TODO : better error handling (search for `unwrap`)

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
	let instructions = cli_parsing::CliInstructions::new();
	let configuration = configuration::Configuration::from(&instructions);

	let mut rng = rand::thread_rng();
	let user_data = user_data::UserData {
		internal_server_port: rng.gen_range(1024, 65535),
		position: 0,
		images: vec![],
		targets: instructions.targets.clone(),
		debug: instructions.debug,
		token: String::new(),
	};
	let arc_user_data: Arc<Mutex<user_data::UserData>> = Arc::new(Mutex::new(user_data));

	webserver::run(instructions.clone(), std::sync::Arc::clone(&arc_user_data));
	window::run(
		instructions.clone(),
		&configuration,
		std::sync::Arc::clone(&arc_user_data),
	);

	if instructions.debug {
		println!("DEBUG: end of program");
	}
}

/* USELESS ?
let mut folders: Vec<String> = vec![];
for entry in std::fs::read_dir(&working_folder).unwrap() {
	let path = entry.unwrap().path();
	if path.is_dir() {
		folders.push(String::from(path.file_name().unwrap().to_str().unwrap()));
	}
}
*/

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz-0123456789_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
