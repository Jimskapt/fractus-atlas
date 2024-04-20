use rand::Rng;
use std::{
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, RwLock},
};
use tauri::Manager;

mod action;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run(init_settings: InitSettings) {
	let (settings_path, settings) = match init_settings {
		InitSettings::File(settings_path) => match std::fs::read_to_string(&settings_path) {
			Ok(content) => match toml::from_str(&content) {
				Ok(settings) => (Some(settings_path), settings),
				Err(err) => {
					eprintln!("{err}");
					(None, common::Settings::default())
				}
			},
			Err(err) => {
				println!(
					"Can not read {} file because : {err}. Using default settings.",
					settings_path.display()
				);
				(Some(settings_path), common::Settings::default())
			}
		},
		InitSettings::Standalone(settings) => (None, settings),
	};

	let mut shortcuts = BTreeMap::new();
	for (id, output) in settings.output_folders.iter().enumerate() {
		for shortcut in &output.shortcuts_or {
			match shortcut {
				common::Shortcut::Key(key) => {
					shortcuts.insert(key.trim().to_lowercase(), action::AppAction::Move(id));
				}
			}
		}
	}
	shortcuts.insert(
		String::from("arrowright"),
		action::AppAction::ChangePosition(1),
	);
	// shortcuts.insert(String::from("d"), action::AppAction::ChangePosition(1));
	shortcuts.insert(
		String::from("arrowleft"),
		action::AppAction::ChangePosition(-1),
	);
	// shortcuts.insert(String::from("q"), action::AppAction::ChangePosition(-1));
	// shortcuts.insert(String::from("backspace"), action::AppAction::RestoreImage);
	shortcuts.insert(String::from(" "), action::AppAction::ChangeRandomPosition);

	let state = Arc::new(RwLock::new(AppState {
		settings_path,
		settings,
		images: vec![],
		current_position: None,
		display_path: String::from("Loading files list"),
		shortcuts,
	}));

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.invoke_handler(tauri::generate_handler![
			keyup,
			get_current_path,
			get_move_actions,
			get_ai_prompt,
			do_move,
			change_path,
			change_position,
			get_current_position,
			get_images_length,
			get_settings,
			set_settings,
			get_settings_path,
			set_settings_path,
			update_files_list
		])
		.manage(state.clone())
		.register_uri_scheme_protocol("image", get_image)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
fn keyup(state: tauri::State<Arc<RwLock<AppState>>>, key: String) -> bool {
	let mut changed = false;

	let processed = key.to_lowercase();
	let shortcuts = state.read().unwrap().shortcuts.clone();

	if let Some(action) = shortcuts.get(&processed) {
		changed = action::apply_action(state.inner().clone(), action)
	}

	changed
}
#[tauri::command]
fn get_current_path(state: tauri::State<Arc<RwLock<AppState>>>) -> String {
	state.read().unwrap().display_path.clone()
}
#[tauri::command]
fn get_move_actions(state: tauri::State<Arc<RwLock<AppState>>>) -> Vec<common::OutputFolder> {
	state.read().unwrap().settings.output_folders.clone()
}
#[tauri::command]
fn do_move(state: tauri::State<Arc<RwLock<AppState>>>, name: String) -> bool {
	if !name.trim().is_empty() {
		let position = state
			.read()
			.unwrap()
			.settings
			.output_folders
			.iter()
			.position(|el| el.name == name);
		if let Some(id) = position {
			action::apply_action(state.inner().clone(), &action::AppAction::Move(id))
		} else {
			// TODO : warn user
			false
		}
	} else {
		action::apply_action(state.inner().clone(), &action::AppAction::RestoreImage)
	}
}
#[tauri::command]
fn change_path(state: tauri::State<Arc<RwLock<AppState>>>, new_path: String) -> bool {
	let mut moved = false;

	let new_pathbuf = std::path::PathBuf::from(new_path);

	let current_position = state.read().unwrap().current_position;
	if let Some(position) = current_position {
		if let Some(current_image) = state.read().unwrap().images.get(position) {
			let old_path = current_image.get_current();

			if old_path.parent().unwrap() == new_pathbuf.parent().unwrap()
				&& old_path.extension() == new_pathbuf.extension()
			{
				match exec_move(&old_path, &new_pathbuf) {
					Ok(_) => {
						moved = true;
					}
					Err(_) => todo!(),
				}
			} else {
				eprintln!("the new specified path has not same parent or file extension than the old path, this is not allowed because of security issue"); // TODO
				return true;
			}
		}
	}

	if moved {
		let steps_after_move = {
			let mut state_w = state.write().unwrap();
			let pos = state_w.current_position.unwrap();
			state_w.images.get_mut(pos).unwrap().moved = Some(new_pathbuf);
			state_w.set_position(current_position);

			state_w.settings.steps_after_move
		};

		return action::apply_action(
			state.inner().clone(),
			&action::AppAction::ChangePosition(steps_after_move),
		);
	} else {
		return false;
	}
}
#[tauri::command]
fn change_position(state: tauri::State<Arc<RwLock<AppState>>>, step: isize) -> bool {
	return action::apply_action(
		state.inner().clone(),
		&action::AppAction::ChangePosition(step),
	);
}
#[tauri::command]
fn get_settings(state: tauri::State<Arc<RwLock<AppState>>>) -> common::Settings {
	state.read().unwrap().settings.clone()
}
#[tauri::command]
fn set_settings_path(
	state: tauri::State<Arc<RwLock<AppState>>>,
	settings_path: String,
) -> Vec<common::SaveMessage> {
	let mut messages = vec![];

	if !settings_path.trim().is_empty() {
		if !std::path::PathBuf::from(&settings_path).exists() {
			state.write().unwrap().settings_path = Some(std::path::PathBuf::from(settings_path));
		} else {
			match std::fs::read_to_string(&settings_path) {
				Ok(value) => match toml::from_str(&value) {
					Ok(new_settings) => {
						let mut data = state.write().unwrap();
						data.settings_path = Some(std::path::PathBuf::from(settings_path));
						data.settings = new_settings;
					}
					Err(err) => {
						messages.push(common::SaveMessage::Warning(format!(
							"can not read `{}` file because : {}",
							settings_path, err
						)));
						state.write().unwrap().settings_path = None;
					}
				},
				Err(err) => {
					messages.push(common::SaveMessage::Warning(format!(
						"can not read `{}` file because : {}",
						settings_path, err
					)));
					state.write().unwrap().settings_path = None;
				}
			}
		}
	} else {
		state.write().unwrap().settings_path = None;
	}

	messages
}
#[tauri::command]
fn set_settings(
	state: tauri::State<Arc<RwLock<AppState>>>,
	new_settings: common::Settings,
) -> Vec<common::SaveMessage> {
	state.write().unwrap().settings = new_settings.clone();

	let mut messages = vec![];

	let maybe_settings_path = state.read().unwrap().settings_path.clone();

	if let Some(settings_path) = maybe_settings_path {
		if !settings_path.exists() {
			if let Err(err) = std::fs::create_dir_all(settings_path.parent().unwrap()) {
				messages.push(common::SaveMessage::Warning(format!(
					"can not create parent folders of `{}` because : {}",
					settings_path.display(),
					err
				)));
			}
		}

		match std::fs::write(
			&settings_path,
			toml::to_string_pretty(&new_settings).unwrap(),
		) {
			Ok(()) => {
				messages.push(common::SaveMessage::Confirm(format!(
					"successfully saved `{}` file",
					settings_path.display()
				)));
			}
			Err(err) => {
				messages.push(common::SaveMessage::Warning(format!(
					"can not write `{}` file because : {}",
					settings_path.display(),
					err
				)));
				state.write().unwrap().settings_path = None;
			}
		}
	}

	messages
}
#[tauri::command]
fn get_settings_path(state: tauri::State<Arc<RwLock<AppState>>>) -> Option<std::path::PathBuf> {
	state.read().unwrap().settings_path.clone()
}
#[tauri::command]
fn update_files_list(state: tauri::State<Arc<RwLock<AppState>>>) {
	let state_for_input = state.inner().clone();
	let folders = state_for_input
		.read()
		.unwrap()
		.settings
		.input_folders
		.clone();

	state_for_input.write().unwrap().images.clear();
	state_for_input.write().unwrap().set_position(None);

	for input in folders {
		let state_for_loop = state.inner().clone();

		tokio::task::spawn(async move {
			let mut temp = tokio::fs::read_dir(&input.path).await.unwrap();
			while let Ok(Some(entry)) = temp.next_entry().await {
				if entry.file_type().await.unwrap().is_file()
					&& input.filter(&input.path.join(entry.file_name()))
				{
					state_for_loop.write().unwrap().images.push(Image {
						origin: input.path.join(entry.file_name()),
						moved: None,
					});

					if !state_for_loop.write().unwrap().images.is_empty()
						&& state_for_loop.read().unwrap().current_position.is_none()
					{
						state_for_loop.write().unwrap().set_position(Some(0));
					} else {
						state_for_loop.write().unwrap().set_position(None);
					}
				}
			}
		});
	}
}
#[tauri::command]
fn get_current_position(state: tauri::State<Arc<RwLock<AppState>>>) -> Option<usize> {
	state.read().unwrap().current_position
}
#[tauri::command]
fn get_images_length(state: tauri::State<Arc<RwLock<AppState>>>) -> usize {
	state.read().unwrap().images.len()
}
#[tauri::command]
fn get_ai_prompt(state: tauri::State<Arc<RwLock<AppState>>>) -> Option<String> {
	let current_position = state.read().unwrap().current_position;

	if let Some(position) = current_position {
		if let Some(image) = state.read().unwrap().images.get(position) {
			let path = image.get_current();

			if let Some(extension) = path.extension() {
				if extension.to_string_lossy().trim().to_lowercase() == "png" {
					if let Ok(bytes) = std::fs::read(&path) {
						return extract_prompt(&bytes);
					}
				}
			}
		}
	}

	return None;
}

fn exec_move(from: &std::path::Path, to: &std::path::Path) -> Result<std::path::PathBuf, String> {
	let mut new_path = to.to_path_buf();

	if from != to {
		while to.exists() {
			let rand_id: String = rand::thread_rng()
				.sample_iter(&rand::distributions::Alphanumeric)
				.take(8)
				.map(char::from)
				.collect();

			new_path = to.parent().unwrap().join(format!(
				"{}{}{}",
				to.file_stem()
					.map(|val| format!("{}-d-", val.to_string_lossy()))
					.unwrap_or_default(),
				rand_id,
				to.extension()
					.map(|val| format!(".{}", val.to_string_lossy()))
					.unwrap_or_default()
			));
		}

		if let Some(parent) = new_path.parent() {
			std::fs::create_dir_all(parent).ok();
		}

		match std::fs::copy(from, &new_path) {
			Ok(_) => {
				if from != new_path {
					if trash::delete(from).is_ok() {
						Ok(new_path)
					} else {
						trash::delete(&new_path).ok();
						// TODO : warn user
						Ok(new_path)
					}
				} else {
					Err(String::from("same path"))
				}
			}
			Err(err) => Err(format!("{err}")),
		}
	} else {
		Ok(to.to_path_buf())
	}
}

fn extract_prompt(bytes: &[u8]) -> Option<String> {
	let mut pos = 0;

	// file signature
	{
		if &bytes[0..4] != b"\x89PNG" {
			return None;
		}

		let block_size: u32 = 8;
		pos += usize::try_from(block_size).unwrap(); // skip block
	}

	// IHDR
	{
		let block_size = u32::from_be_bytes(<[u8; 4]>::try_from(&bytes[pos..pos + 4]).unwrap());
		pos += 4;

		// let block_name = std::str::from_utf8(&bytes[pos..pos+4]).unwrap();
		pos += 4;

		pos += usize::try_from(block_size).unwrap(); // skip block

		// let crc = u32::from_be_bytes(<[u8; 4]>::try_from(&bytes[pos..pos+4]).unwrap());
		pos += 4;
	}

	// tEXt
	{
		let start_text = pos;
		let block_size = u32::from_be_bytes(<[u8; 4]>::try_from(&bytes[pos..pos + 4]).unwrap());
		pos += 4;

		let block_name = std::str::from_utf8(&bytes[pos..pos + 4]).unwrap();
		pos += 4;

		if block_name == "tEXt" || block_name == "iEXt" {
			// return Some(String::from(std::str::from_utf8(&bytes[pos..(pos+(block_size as usize)-1)]).unwrap()));

			let mut remaining = vec![];
			remaining.extend_from_slice(&bytes[..start_text]);
			remaining.extend_from_slice(&bytes[(pos + (block_size as usize) + 4)..]);

			let text = &bytes[pos..(pos + (block_size as usize) - 1)];
			let decoded_text = if block_name == "tEXt" {
				let (decoded_value, _has_malformed) =
					encoding_rs::WINDOWS_1252.decode_with_bom_removal(text);
				String::from(decoded_value)
			} else {
				String::from_utf8_lossy(text).into_owned()
			};

			if let Some(value) = decoded_text.strip_prefix("parameters\x00") {
				return Some(String::from(value));
			}
		}
	}

	return None;
}

fn get_image(
	app: &tauri::AppHandle,
	_request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
	let app_state: tauri::State<Arc<RwLock<AppState>>> = app.state();
	let app_state_read = app_state.inner().read().unwrap();

	tauri::http::Response::builder()
		/*
		.header("Content-Type", "image/png")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Methods", "GET, OPTIONS")
		.header("Access-Control-Allow-Headers", "Content-Type")
		*/
		.body(match &app_state_read.current_position {
			Some(position) => match app_state_read.images.get(*position) {
				Some(image) => match std::fs::read(image.get_current()) {
					Ok(content) => content.to_vec(),
					Err(_) => {
						include_bytes!("../../../src-front/assets/undraw_access_denied_re_awnf.png")
							.to_vec()
					}
				},
				None => {
					include_bytes!("../../../src-front/assets/undraw_Page_not_found_re_e9o6.png")
						.to_vec()
				}
			},
			None => include_bytes!("../../../src-front/assets/undraw_Loading_re_5axr.png").to_vec(),
		})
		.unwrap()
}

pub enum InitSettings {
	Standalone(common::Settings),
	File(PathBuf),
}

pub struct AppState {
	settings_path: Option<PathBuf>,
	settings: common::Settings,
	images: Vec<Image>,
	current_position: Option<usize>,
	display_path: String,
	shortcuts: BTreeMap<String, action::AppAction>,
}
impl AppState {
	pub fn set_position(&mut self, new_position: Option<usize>) {
		self.current_position = new_position;

		match new_position {
			Some(new_position_usize) => {
				if let Some(image) = self.images.get(new_position_usize) {
					self.display_path = format!("{}", image.get_current().display());
				} else {
					self.display_path = String::from("error : unknown position");
				}
			}
			None => {
				self.display_path = String::from(
					"No image, to the moment. Probably because target folders are empty.",
				);
			}
		}
	}
}

#[derive(Debug, Clone)]
struct Image {
	origin: PathBuf,
	moved: Option<PathBuf>,
}
impl Image {
	pub fn get_current(&self) -> PathBuf {
		if let Some(path) = &self.moved {
			path.clone()
		} else {
			self.origin.clone()
		}
	}
}
