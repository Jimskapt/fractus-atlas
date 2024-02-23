// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

#[tokio::main]
async fn main() {
	let settings_path = PathBuf::from(format!("./{}.conf.toml", env!("CARGO_PKG_NAME")));

	// std::fs::write(&settings_path, toml::to_string(&fractus_atlas_lib::Settings::default()).unwrap()).unwrap();

	fractus_atlas_lib::run(fractus_atlas_lib::InitSettings::File(settings_path)).await;
}
