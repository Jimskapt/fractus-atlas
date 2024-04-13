// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
	let mut args = std::env::args();

	let run_path = std::path::PathBuf::from(args.next().unwrap());
	let run_path_parent = run_path.parent().unwrap();

	let settings_path = args
		.next()
		.unwrap_or_else(|| format!("{}.conf.toml", env!("CARGO_PKG_NAME")));
	let settings_pathbuf = std::path::PathBuf::from(settings_path);

	let absolute_settings_path = if settings_pathbuf.is_absolute() {
		settings_pathbuf
	} else {
		run_path_parent.join(settings_pathbuf)
	};

	fractus_atlas_lib::run(fractus_atlas_lib::InitSettings::File(
		absolute_settings_path,
	))
	.await;
}
