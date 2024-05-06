use rand::Rng;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::RwLock;

static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

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

	let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

	let state = Arc::new(RwLock::new(AppState {
		settings_path,
		settings,
		images: vec![],
		current_position: None,
		display_path: String::from("Loading files list"),
		shortcuts,
		watcher: None,
	}));

	let sender_for_watcher = sender.clone();
	let watcher = <notify::RecommendedWatcher as notify::Watcher>::new(
		move |res: Result<notify::Event, notify::Error>| match res {
			Ok(event) => {
				if let notify::EventKind::Create(_) = event.kind {
					if let Some(path) = event.paths.first() {
						let sender_for_task = sender_for_watcher.clone();
						let path_for_task = path.clone();
						futures::executor::block_on(async {
							sender_for_task
								.send(FileEvent::Add(path_for_task))
								.await
								.unwrap();
						});
					}
				} else if let notify::EventKind::Modify(notify::event::ModifyKind::Name(
					notify::event::RenameMode::To,
				)) = event.kind
				{
					if let Some(path) = event.paths.first() {
						let sender_for_task = sender_for_watcher.clone();
						let path_for_task = path.clone();
						futures::executor::block_on(async {
							sender_for_task
								.send(FileEvent::Rename(path_for_task))
								.await
								.unwrap();
						});
					}
				}
			}
			Err(_) => todo!(),
		},
		notify::Config::default(),
	)
	.unwrap();

	state.write().await.watcher = Some(watcher);

	let state_for_file_channel = state.clone();
	let sort = state.clone().read().await.settings.sorting.clone();
	tokio::task::spawn(async move {
		while let Some(event) = receiver.recv().await {
			match event {
				FileEvent::Add(add_path) => {
					println!("{}", add_path.display());
					receive(state_for_file_channel.clone(), add_path).await
				},
				FileEvent::Rename(add_path) => {
					receive(state_for_file_channel.clone(), add_path).await
				}
			}

			match sort {
				common::SortingOrder::FileName => {
					state_for_file_channel
						.write()
						.await
						.images
						.sort_by(|a, b| a.origin.cmp(&b.origin));
				}
			}
		}
	});

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.setup(|app| {
			let handle = app.app_handle().clone();

			/*
			app.listen_any("request-ui-refresh", |event| {
				println!("got request-ui-refresh with payload {:?}", event.payload());
			});
			*/

			_ = APP_HANDLE.set(handle);

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			keyup,
			get_current_path,
			get_move_actions,
			get_ai_prompt,
			do_move,
			change_path,
			change_position,
			set_random_position,
			get_current_position,
			get_images_length,
			current_can_be_restored,
			get_settings,
			set_settings,
			get_settings_path,
			set_settings_path,
			os_open,
			update_files_list,
			is_confirm_rename,
			get_backend_version,
		])
		.manage(state.clone())
		.manage(sender)
		.register_asynchronous_uri_scheme_protocol("image", |app, request, responder| {
			let app_for_async = app.clone();
			tauri::async_runtime::spawn(async move {
				let response = get_image(&app_for_async, request).await;
				responder.respond(response)
			});
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

#[tauri::command]
async fn keyup(state: tauri::State<'_, Arc<RwLock<AppState>>>, key: String) -> Result<bool, ()> {
	let mut changed = Ok(false);

	let processed = key.to_lowercase();
	let shortcuts = state.read().await.shortcuts.clone();

	if let Some(action) = shortcuts.get(&processed) {
		changed = action::apply_action(state.inner().clone(), action).await
	}

	changed
}
#[tauri::command]
async fn get_current_path(state: tauri::State<'_, Arc<RwLock<AppState>>>) -> Result<String, ()> {
	Ok(state.read().await.display_path.clone())
}
#[tauri::command]
async fn get_move_actions(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<common::OutputFolder>, ()> {
	Ok(state.read().await.settings.output_folders.clone())
}
#[tauri::command]
async fn do_move(state: tauri::State<'_, Arc<RwLock<AppState>>>, name: String) -> Result<bool, ()> {
	if !name.trim().is_empty() {
		let position = state
			.read()
			.await
			.settings
			.output_folders
			.iter()
			.position(|el| el.name == name);
		if let Some(id) = position {
			action::apply_action(state.inner().clone(), &action::AppAction::Move(id)).await
		} else {
			// TODO : warn user
			Ok(false)
		}
	} else {
		action::apply_action(state.inner().clone(), &action::AppAction::RestoreImage).await
	}
}
#[tauri::command]
async fn change_path(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	new_path: String,
) -> Result<bool, ()> {
	let mut moved = false;

	let new_pathbuf = std::path::PathBuf::from(new_path);

	let current_position = state.read().await.current_position;
	if let Some(position) = current_position {
		if let Some(current_image) = state.read().await.images.get(position) {
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
				return Ok(true);
			}
		}
	}

	if moved {
		let steps_after_move = {
			let mut state_w = state.write().await;
			let pos = state_w.current_position.unwrap();
			state_w.images.get_mut(pos).unwrap().moved = Some(new_pathbuf);
			state_w.set_position(current_position);

			state_w.settings.steps_after_move
		};

		return action::apply_action(
			state.inner().clone(),
			&action::AppAction::ChangePosition(steps_after_move),
		)
		.await;
	} else {
		return Ok(false);
	}
}
#[tauri::command]
async fn change_position(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	step: isize,
) -> Result<bool, ()> {
	return action::apply_action(
		state.inner().clone(),
		&action::AppAction::ChangePosition(step),
	)
	.await;
}
#[tauri::command]
async fn set_random_position(state: tauri::State<'_, Arc<RwLock<AppState>>>) -> Result<bool, ()> {
	return action::apply_action(
		state.inner().clone(),
		&action::AppAction::ChangeRandomPosition,
	)
	.await;
}
#[tauri::command]
async fn get_settings(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<common::Settings, ()> {
	Ok(state.read().await.settings.clone())
}
#[tauri::command]
async fn set_settings_path(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	settings_path: String,
) -> Result<Vec<common::SaveMessage>, ()> {
	let mut messages = vec![];

	if !settings_path.trim().is_empty() {
		if !std::path::PathBuf::from(&settings_path).exists() {
			state.write().await.settings_path = Some(std::path::PathBuf::from(settings_path));
		} else {
			match std::fs::read_to_string(&settings_path) {
				Ok(value) => match toml::from_str(&value) {
					Ok(new_settings) => {
						let mut data = state.write().await;
						data.settings_path = Some(std::path::PathBuf::from(settings_path));
						data.settings = new_settings;
					}
					Err(err) => {
						messages.push(common::SaveMessage::Warning(format!(
							"can not read `{}` file because : {}",
							settings_path, err
						)));
						state.write().await.settings_path = None;
					}
				},
				Err(err) => {
					messages.push(common::SaveMessage::Warning(format!(
						"can not read `{}` file because : {}",
						settings_path, err
					)));
					state.write().await.settings_path = None;
				}
			}
		}
	} else {
		state.write().await.settings_path = None;
	}

	Ok(messages)
}
#[tauri::command]
async fn set_settings(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	new_settings: common::Settings,
) -> Result<Vec<common::SaveMessage>, ()> {
	let mut modified_settings = new_settings.clone();
	modified_settings.settings_version = Some(String::from(env!("CARGO_PKG_VERSION")));

	state.write().await.settings = modified_settings.clone();

	let mut messages = vec![];

	let maybe_settings_path = state.read().await.settings_path.clone();

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
			toml::to_string_pretty(&modified_settings).unwrap(),
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
				state.write().await.settings_path = None;
			}
		}
	}

	Ok(messages)
}
#[tauri::command]
async fn get_settings_path(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Option<std::path::PathBuf>, ()> {
	Ok(state.read().await.settings_path.clone())
}
#[tauri::command]
async fn update_files_list(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	sender: tauri::State<'_, tokio::sync::mpsc::Sender<FileEvent>>,
) -> Result<(), ()> {
	let state_for_input = state.inner().clone();
	let folders = state_for_input.read().await.settings.input_folders.clone();

	for input in &folders {
		notify::Watcher::unwatch(
			state_for_input.write().await.watcher.as_mut().unwrap(),
			&input.path,
		)
		.ok();
	}

	state_for_input.write().await.images.clear();
	state_for_input.write().await.set_position(None);

	for input in folders {
		let state_for_task = state_for_input.clone();
		let sender_for_task = (*sender).clone();
		tokio::task::spawn(async move {
			browse_dir(
				&input.path,
				sender_for_task.clone(),
				input.recursivity.unwrap_or(false),
			)
			.await;

			notify::Watcher::watch(
				state_for_task.write().await.watcher.as_mut().unwrap(),
				&input.path,
				if input.recursivity.unwrap_or(false) {
					notify::RecursiveMode::Recursive
				} else {
					notify::RecursiveMode::NonRecursive
				},
			)
			.unwrap();
		});
	}

	Ok(())
}
#[tauri::command]
async fn get_current_position(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Option<usize>, ()> {
	Ok(state.read().await.current_position)
}
#[tauri::command]
async fn get_images_length(state: tauri::State<'_, Arc<RwLock<AppState>>>) -> Result<usize, ()> {
	Ok(state.read().await.images.len())
}
#[tauri::command]
async fn get_ai_prompt(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Option<String>, ()> {
	let current_position = state.read().await.current_position;

	if let Some(position) = current_position {
		if let Some(image) = state.read().await.images.get(position) {
			let path = image.get_current();

			if let Some(extension) = path.extension() {
				if extension.to_string_lossy().trim().to_lowercase() == "png" {
					if let Ok(bytes) = std::fs::read(&path) {
						return Ok(extract_prompt(&bytes));
					}
				}
			}
		}
	}

	return Ok(None);
}
#[tauri::command]
async fn current_can_be_restored(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<bool, ()> {
	match state.read().await.current_position {
		Some(position) => Ok(state
			.read()
			.await
			.images
			.get(position)
			.unwrap()
			.moved
			.is_some()),
		None => Ok(false),
	}
}
#[tauri::command]
async fn os_open(
	state: tauri::State<'_, Arc<RwLock<AppState>>>,
	open_target: String,
) -> Result<(), ()> {
	if let Some(position) = state.read().await.current_position {
		let target_path = state
			.read()
			.await
			.images
			.get(position)
			.unwrap()
			.get_current();
		if open_target == "file" {
			open::that(target_path).unwrap();
		} else {
			open::that(target_path.parent().unwrap()).unwrap();
		}
	}

	Ok(())
}
#[tauri::command]
async fn is_confirm_rename(state: tauri::State<'_, Arc<RwLock<AppState>>>) -> Result<bool, ()> {
	Ok(state.read().await.settings.confirm_rename.unwrap_or(true))
}
#[tauri::command]
fn get_backend_version() -> String {
	String::from(env!("CARGO_PKG_VERSION"))
}

#[async_recursion::async_recursion]
async fn browse_dir(
	path: &std::path::Path,
	sender: tokio::sync::mpsc::Sender<FileEvent>,
	recursivity: bool,
) {
	let mut temp = tokio::fs::read_dir(&path).await.unwrap();
	while let Ok(Some(entry)) = temp.next_entry().await {
		if entry.path().is_file() {
			sender
				.send(FileEvent::Add(path.join(entry.file_name())))
				.await
				.unwrap();
		} else if entry.path().is_dir() && recursivity {
			browse_dir(&path.join(entry.file_name()), sender.clone(), recursivity).await;
		}
	}
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

async fn get_image(
	app: &tauri::AppHandle,
	_request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
	let app_state: tauri::State<Arc<RwLock<AppState>>> = app.state();
	let app_state_read = app_state.inner().read().await;

	tauri::http::Response::builder()
		/*
		.header("Content-Type", "image/png")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Methods", "GET, OPTIONS")
		.header("Access-Control-Allow-Headers", "Content-Type")
		*/
		.body(match &app_state_read.current_position {
			Some(position) => match app_state_read.images.get(*position) {
				Some(image) => {
					let bytes = std::fs::read(image.get_current());
					match bytes {
						Ok(content) => content.to_vec(),
						Err(_) => include_bytes!(
							"../../../src-front/assets/undraw_access_denied_re_awnf.png"
						)
						.to_vec(),
					}
				}
				None => {
					include_bytes!("../../../src-front/assets/undraw_Page_not_found_re_e9o6.png")
						.to_vec()
				}
			},
			None => include_bytes!("../../../src-front/assets/undraw_Loading_re_5axr.png").to_vec(),
		})
		.unwrap()
}

async fn receive(state: Arc<RwLock<AppState>>, add_path: std::path::PathBuf) {
	let input_folders = state.write().await.settings.input_folders.clone();
	'inputs: for input_folder in input_folders {
		if add_path.is_file()
			&& input_folder.filter(&add_path)
			&& add_path.starts_with(input_folder.path)
		{
			{
				let images = &mut state.write().await.images;

				if !images.iter().any(|el| {
					if el.origin == add_path {
						true
					} else if let Some(moved) = &el.moved {
						moved == &add_path
					} else {
						false
					}
				}) {
					images.push(Image {
						origin: add_path.clone(),
						moved: None,
					});
				}
			}

			let none_position = state.read().await.current_position.is_none();
			if none_position {
				state.write().await.set_position(Some(0));
			}

			APP_HANDLE
				.get()
				.unwrap()
				.emit("request-ui-refresh", add_path)
				.unwrap();

			break 'inputs;
		}
	}
}

pub enum InitSettings {
	Standalone(common::Settings),
	File(PathBuf),
}

#[derive(Debug)]
enum FileEvent {
	Add(std::path::PathBuf),
	Rename(std::path::PathBuf),
}

pub struct AppState {
	settings_path: Option<PathBuf>,
	settings: common::Settings,
	images: Vec<Image>,
	current_position: Option<usize>,
	display_path: String,
	shortcuts: BTreeMap<String, action::AppAction>,
	watcher: Option<notify::RecommendedWatcher>,
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
