use rand::Rng;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tauri::{Emitter, Listener, Manager};
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
					// eprintln!("{err}");
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

	let global_log_path =
		dirs::data_local_dir().map(|path| path.join(env!("CARGO_PKG_NAME")).join("global.log"));

	let stdout = log4rs::append::console::ConsoleAppender::builder()
		.target(log4rs::append::console::Target::Stdout)
		.build();

	let level_filter = log::LevelFilter::Trace;

	let config = if let Some(global_log) = global_log_path {
		let global_logger = log4rs::append::file::FileAppender::builder()
			.encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
				"{d} - {m}{n}",
			)))
			.build(global_log)
			.unwrap();

		log4rs::config::Config::builder()
			.appender(
				log4rs::config::Appender::builder().build("globalfile", Box::new(global_logger)),
			)
			.appender(
				log4rs::config::Appender::builder()
					.filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
						level_filter,
					)))
					.build("stdout", Box::new(stdout)),
			)
			.build(
				log4rs::config::Root::builder()
					.appender("globalfile")
					.appender("stdout")
					.build(level_filter),
			)
			.unwrap()
	} else {
		log4rs::config::Config::builder()
			.appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
			.logger(log4rs::config::Logger::builder().build("stdout", level_filter))
			.build(
				log4rs::config::Root::builder()
					.appender("stdout")
					.build(level_filter),
			)
			.unwrap()
	};

	let _handle = log4rs::init_config(config).unwrap();

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

	let state = Arc::new(AppState {
		settings_path: RwLock::new(settings_path),
		settings: RwLock::new(settings),
		images: RwLock::new(Images::default()),
		shortcuts: RwLock::new(shortcuts),
		watcher: RwLock::new(None),
		refresh: RwLock::new(false),
	});

	let state_for_watcher = state.clone();
	let watcher: notify::ReadDirectoryChangesWatcher =
		<notify::RecommendedWatcher as notify::Watcher>::new(
			move |res: Result<notify::Event, notify::Error>| {
				futures::executor::block_on(async {
					log::debug!("new watcher event : {res:?}");

					match res {
						Ok(event) => {
							if let notify::EventKind::Create(_) = event.kind {
								if let Some(path) = event.paths.first() {
									let path_for_task = path.clone();

									let state_settings = state_for_watcher.settings.read().await;
									let input_folders = state_settings.input_folders.as_slice();
									let state_settings_path =
										state_for_watcher.settings_path.read().await.clone();

									if input_folders.iter().any(|folder| {
										folder.filter(path, state_settings_path.clone())
									}) {
										log::trace!("trying write PLZ-965");
										state_for_watcher.images.write().await.push(
											path_for_task,
											&mut *state_for_watcher.refresh.write().await,
											&state_settings,
										);
										log::trace!("finished write PLZ-965");
									}
								}
							} else if let notify::EventKind::Modify(
								notify::event::ModifyKind::Name(notify::event::RenameMode::To),
							) = event.kind
							{
								if let Some(path) = event.paths.first() {
									let path_for_task = path.clone();

									let state_settings = state_for_watcher.settings.read().await;
									let input_folders = state_settings.input_folders.as_slice();
									let state_settings_path =
										state_for_watcher.settings_path.read().await.clone();

									if input_folders.iter().any(|folder| {
										folder.filter(path, state_settings_path.clone())
									}) {
										log::trace!("trying write TGS-951");
										state_for_watcher.images.write().await.push(
											path_for_task,
											&mut *state_for_watcher.refresh.write().await,
											&state_settings,
										);
										log::trace!("finished write TGS-951");
									}
								}
							}
						}
						Err(_) => todo!(),
					}
				})
			},
			notify::Config::default(),
		)
		.unwrap();

	*state.clone().watcher.write().await = Some(watcher);

	let state_for_task = state.clone();
	tokio::task::spawn(async move {
		loop {
			let refresh = *state_for_task.refresh.read().await;
			if refresh {
				APP_HANDLE
					.get()
					.unwrap()
					.emit("request-ui-refresh", ())
					.unwrap();

				*state_for_task.refresh.write().await = false;
				tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
			}
		}
	});

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.setup(|app| {
			let handle = app.app_handle().clone();

			app.listen_any("request-ui-refresh", |event| {
				log::debug!("got request-ui-refresh with payload {:?}", event.payload());
			});

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
		.manage(state)
		.register_asynchronous_uri_scheme_protocol("image", |ctx, request, responder| {
			let app = ctx.app_handle();
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
async fn keyup(state: tauri::State<'_, Arc<AppState>>, key: String) -> Result<bool, ()> {
	let mut changed = Ok(false);

	let processed = key.to_lowercase();

	if let Some(action) = state.shortcuts.read().await.get(&processed) {
		changed = action::apply_action(state.inner(), action).await;
	}

	changed
}
#[tauri::command]
async fn get_current_path(state: tauri::State<'_, Arc<AppState>>) -> Result<String, ()> {
	Ok(state.images.read().await.display_path())
}
#[tauri::command]
async fn get_move_actions(
	state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<common::OutputFolder>, ()> {
	Ok(state.settings.read().await.output_folders.clone())
}
#[tauri::command]
async fn do_move(state: tauri::State<'_, Arc<AppState>>, name: String) -> Result<bool, ()> {
	let res = if !name.trim().is_empty() {
		let position = state
			.settings
			.read()
			.await
			.output_folders
			.iter()
			.position(|el| el.name == name);
		if let Some(id) = position {
			action::apply_action(state.inner(), &action::AppAction::Move(id)).await
		} else {
			// TODO : warn user
			Ok(false)
		}
	} else {
		action::apply_action(state.inner(), &action::AppAction::RestoreImage).await
	};

	log::trace!("res = {res:?}");

	res
}
#[tauri::command]
async fn change_path(state: tauri::State<'_, Arc<AppState>>, new_path: String) -> Result<bool, ()> {
	let mut moved = false;

	let new_pathbuf = std::path::PathBuf::from(new_path);

	if let Some(current_image) = state.images.read().await.get_current() {
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

	if moved {
		{
			log::trace!("trying write NFB-974");
			if let Err(err) = state
				.images
				.write()
				.await
				.move_current(new_pathbuf, &mut *state.refresh.write().await)
			{
				log::error!("error while save new current path : {err}");
			}
			log::trace!("finished write NFB-974");
		}
		// TODO : usefull ? state.set_position(current_position);

		let steps_after_move = state.settings.read().await.steps_after_move;

		return action::apply_action(
			state.inner(),
			&action::AppAction::ChangePosition(steps_after_move),
		)
		.await;
	} else {
		return Ok(false);
	}
}
#[tauri::command]
async fn change_position(state: tauri::State<'_, Arc<AppState>>, step: isize) -> Result<bool, ()> {
	return action::apply_action(state.inner(), &action::AppAction::ChangePosition(step)).await;
}
#[tauri::command]
async fn set_random_position(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, ()> {
	return action::apply_action(state.inner(), &action::AppAction::ChangeRandomPosition).await;
}
#[tauri::command]
async fn get_settings(state: tauri::State<'_, Arc<AppState>>) -> Result<common::Settings, ()> {
	Ok(state.settings.read().await.clone())
}
#[tauri::command]
async fn set_settings_path(
	state: tauri::State<'_, Arc<AppState>>,
	settings_path: String,
) -> Result<Vec<common::SaveMessage>, ()> {
	let mut messages = vec![];

	if !settings_path.trim().is_empty() {
		if !std::path::PathBuf::from(&settings_path).exists() {
			log::trace!("trying write MGF-856");
			*state.settings_path.write().await = Some(std::path::PathBuf::from(settings_path));
			log::trace!("finished write MGF-856");
		} else {
			match std::fs::read_to_string(&settings_path) {
				Ok(value) => match toml::from_str(&value) {
					Ok(new_settings) => {
						{
							log::trace!("trying write PLT-884");
							*state.settings_path.write().await =
								Some(std::path::PathBuf::from(settings_path));
							log::trace!("finished write PLT-884");
						}
						{
							log::trace!("trying write JDB-637");
							*state.settings.write().await = new_settings;
							log::trace!("finished write JDB-637");
						}
					}
					Err(err) => {
						messages.push(common::SaveMessage::Warning(format!(
							"can not read `{}` file because : {}",
							settings_path, err
						)));
						{
							log::trace!("trying write RNY-678");
							*state.settings_path.write().await = None;
							log::trace!("finished write RNY-678");
						}
					}
				},
				Err(err) => {
					messages.push(common::SaveMessage::Warning(format!(
						"can not read `{}` file because : {}",
						settings_path, err
					)));
					{
						log::trace!("trying write CFN-314");
						*state.settings_path.write().await = None;
						log::trace!("finished write CFN-314");
					}
				}
			}
		}
	} else {
		log::trace!("trying write MFB-554");
		*state.settings_path.write().await = None;
		log::trace!("finished write MFB-554");
	}

	Ok(messages)
}
#[tauri::command]
async fn set_settings(
	state: tauri::State<'_, Arc<AppState>>,
	new_settings: common::Settings,
) -> Result<Vec<common::SaveMessage>, ()> {
	let mut modified_settings = new_settings.clone();
	modified_settings.settings_version = Some(String::from(env!("CARGO_PKG_VERSION")));

	{
		log::trace!("trying write DJG-142");
		*state.settings.write().await = modified_settings.clone();
		log::trace!("finished write DJG-142");
	}

	let mut messages = vec![];

	let maybe_settings_path = state.settings_path.read().await.clone();

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
				{
					log::trace!("trying write ZHC-645");
					*state.settings_path.write().await = None;
					log::trace!("finished write ZHC-645");
				}
			}
		}
	}

	Ok(messages)
}
#[tauri::command]
async fn get_settings_path(
	state: tauri::State<'_, Arc<AppState>>,
) -> Result<Option<std::path::PathBuf>, ()> {
	Ok(state.settings_path.read().await.clone())
}
#[tauri::command]
async fn update_files_list(state: tauri::State<'_, Arc<AppState>>) -> Result<(), ()> {
	let folders = &state.inner().settings.read().await.input_folders;
	let settings_path = state.inner().settings_path.read().await;

	let absolute_folders: Vec<common::InputFolder> = folders
		.iter()
		.map(|folder| {
			let mut clone = folder.clone();

			clone.path = common::build_absolute_path(clone.path, settings_path.clone());

			clone
		})
		.collect();

	for input in &absolute_folders {
		log::trace!("trying write HSO-497");
		log::trace!("unwatch {}", input.path.display());
		notify::Watcher::unwatch(
			state.inner().watcher.write().await.as_mut().unwrap(),
			&input.path,
		)
		.ok();
		log::trace!("finished write HSO-497");
	}

	{
		log::trace!("trying write LEV-871");
		(*state.inner().images.write().await).clear(&mut *state.refresh.write().await);
		log::trace!("finished write LEV-871");
	}
	{
		log::trace!("trying write YRB-446");
		state
			.inner()
			.images
			.write()
			.await
			.set_position(None, &mut *state.refresh.write().await);
		log::trace!("finished write YRB-446");
	}

	for input in folders {
		let input_path = common::build_absolute_path(
			&input.path,
			state.inner().settings_path.read().await.clone(),
		);

		log::trace!("started browsing {:?}", input_path);

		browse_dir(
			state.inner(),
			&input_path,
			input.recursivity.unwrap_or(false),
		)
		.await;

		log::trace!("finished browsing {:?}", input_path);

		log::trace!("trying write TJK-766");
		log::trace!("watch {}", input_path.display());
		notify::Watcher::watch(
			state.inner().watcher.write().await.as_mut().unwrap(),
			&input_path,
			if input.recursivity.unwrap_or(false) {
				notify::RecursiveMode::Recursive
			} else {
				notify::RecursiveMode::NonRecursive
			},
		)
		.unwrap();
		log::trace!("finished write TJK-766");
	}

	Ok(())
}
#[tauri::command]
async fn get_current_position(state: tauri::State<'_, Arc<AppState>>) -> Result<Option<usize>, ()> {
	Ok(state.images.read().await.get_current_pos())
}
#[tauri::command]
async fn get_images_length(state: tauri::State<'_, Arc<AppState>>) -> Result<usize, ()> {
	Ok(state.images.read().await.len())
}
#[tauri::command]
async fn get_ai_prompt(state: tauri::State<'_, Arc<AppState>>) -> Result<Option<String>, ()> {
	if let Some(image) = state.images.read().await.get_current() {
		let path = image.get_current();

		if let Some(extension) = path.extension() {
			if extension.to_string_lossy().trim().to_lowercase() == "png" {
				if let Ok(bytes) = std::fs::read(&path) {
					return Ok(extract_prompt(&bytes));
				}
			}
		}
	}

	return Ok(None);
}
#[tauri::command]
async fn current_can_be_restored(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, ()> {
	match state.images.read().await.get_current() {
		Some(image) => Ok(image.moved.is_some()),
		None => Ok(false),
	}
}
#[tauri::command]
async fn os_open(state: tauri::State<'_, Arc<AppState>>, open_target: String) -> Result<(), ()> {
	if let Some(path) = state
		.images
		.read()
		.await
		.get_current()
		.map(|image| image.get_current())
	{
		let target_path = if open_target == "file" {
			path
		} else {
			path.parent().unwrap().to_path_buf()
		};

		log::debug!("trying to open {target_path:?}");
		open::that_detached(target_path).unwrap();
	}

	Ok(())
}
#[tauri::command]
async fn is_confirm_rename(state: tauri::State<'_, Arc<AppState>>) -> Result<bool, ()> {
	Ok(state.settings.read().await.confirm_rename.unwrap_or(true))
}
#[tauri::command]
fn get_backend_version() -> String {
	String::from(env!("CARGO_PKG_VERSION"))
}

#[async_recursion::async_recursion]
async fn browse_dir(state: &AppState, path: &std::path::Path, recursivity: bool) {
	let input_folders = state.settings.read().await.input_folders.clone();
	let settings_path = state.settings_path.read().await.clone();

	let absolute_path = common::build_absolute_path(path, settings_path.clone());

	let mut temp = tokio::fs::read_dir(&absolute_path).await.unwrap();
	let settings_path_for_loop = settings_path.clone();
	while let Ok(Some(entry)) = temp.next_entry().await {
		let merged_path = absolute_path.join(entry.file_name());

		if entry.path().is_file() {
			let settings_path_for_filters = settings_path_for_loop.clone();
			if input_folders
				.iter()
				.any(|folder| folder.filter(&merged_path, settings_path_for_filters.clone()))
			{
				log::debug!("add : {:?}", merged_path);

				{
					log::trace!("trying write OGR-227");
					state.images.write().await.push(
						merged_path,
						&mut *state.refresh.write().await,
						&state.settings.read().await.clone(),
					);
					log::trace!("finished write OGR-227");
				}
			} else {
				log::debug!("do not match input_folders : {:?}", merged_path);
			}
		} else if entry.path().is_dir() && recursivity {
			browse_dir(state, &merged_path, recursivity).await;
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
	let app_state: tauri::State<Arc<AppState>> = app.state();
	let images = app_state.images.read().await;

	tauri::http::Response::builder()
		/*
		.header("Content-Type", "image/png")
		.header("Access-Control-Allow-Origin", "*")
		.header("Access-Control-Allow-Methods", "GET, OPTIONS")
		.header("Access-Control-Allow-Headers", "Content-Type")
		*/
		.body(match images.get_current() {
			Some(image) => {
				let bytes = std::fs::read(image.get_current());
				match bytes {
					Ok(content) => content.to_vec(),
					Err(_) => {
						include_bytes!("../../../src-front/assets/undraw_access_denied_re_awnf.png")
							.to_vec()
					}
				}
			}
			None => include_bytes!("../../../src-front/assets/undraw_Page_not_found_re_e9o6.png")
				.to_vec(),
		})
		.unwrap()
}

pub enum InitSettings {
	Standalone(common::Settings),
	File(PathBuf),
}

pub struct AppState {
	settings_path: RwLock<Option<PathBuf>>,
	settings: RwLock<common::Settings>,
	images: RwLock<Images>,
	shortcuts: RwLock<BTreeMap<String, action::AppAction>>,
	watcher: RwLock<Option<notify::RecommendedWatcher>>,
	refresh: RwLock<bool>,
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

#[derive(Debug, Default)]
struct Images {
	data: Vec<Image>,
	position: Option<usize>,
}

impl Images {
	pub fn push(
		&mut self,
		new: impl Into<PathBuf>,
		state_refresh: &mut bool,
		settings: &common::Settings,
	) -> Option<PathBuf> {
		let new_path = new.into();

		if !self
			.data
			.iter()
			.any(|image| image.origin == new_path || image.moved == Some(new_path.clone()))
		{
			let on_last = self.position == Some(self.data.len().saturating_sub(1));

			self.data.push(Image {
				origin: new_path,
				moved: None,
			});

			if self.position.is_none() {
				self.set_position(Some(0), state_refresh);
			} else if on_last && settings.move_to_newest.unwrap_or(false) {
				self.set_position(Some(self.data.len().saturating_sub(1)), state_refresh);
			}

			*state_refresh = true;

			None
		} else {
			Some(new_path)
		}
	}

	pub fn set_position(&mut self, new_position: Option<usize>, state_refresh: &mut bool) {
		self.position = new_position;

		*state_refresh = true;
	}

	pub fn clear(&mut self, state_refresh: &mut bool) {
		self.data.clear();
		self.position = None;

		*state_refresh = true;
	}

	pub fn move_current(
		&mut self,
		requested_path: impl Into<PathBuf>,
		state_refresh: &mut bool,
	) -> Result<(), String> {
		if let Some(current) = self.get_current() {
			let new_path = requested_path.into();
			let move_res = exec_move(&current.get_current(), &new_path);

			if move_res.is_ok() {
				let pos = self
					.get_current_pos()
					.ok_or(String::from("current position is not set"))?;
				self.data
					.get_mut(pos)
					.ok_or(String::from("can not get mutable data"))?
					.moved = Some(new_path);

				*state_refresh = true;

				Ok(())
			} else {
				move_res.map(|_| ())
			}
		} else {
			Err(String::from("no current yet"))
		}
	}

	pub fn restore_current(&mut self, state_refresh: &mut bool) -> Result<(), String> {
		if let Some(current) = self.get_current() {
			let current_path = current.get_current();
			let origin_path = current.origin.clone();

			let move_res = exec_move(&current_path, &origin_path);
			if move_res.is_ok() {
				let pos = self
					.get_current_pos()
					.ok_or(String::from("current position is not set"))?;
				self.data
					.get_mut(pos)
					.ok_or(String::from("can not get mutable data"))?
					.moved = None;

				*state_refresh = true;

				Ok(())
			} else {
				move_res.map(|_| ())
			}
		} else {
			Err(String::from("no current, yet"))
		}
	}
}

impl Images {
	pub fn get_pos(&self, pos: usize) -> Option<&Image> {
		self.data.get(pos)
	}

	pub fn get_current(&self) -> Option<&Image> {
		match self.position {
			Some(pos) => self.get_pos(pos),
			None => None,
		}
	}

	pub fn display_path(&self) -> String {
		match self.get_current() {
			Some(image) => format!("{}", image.get_current().display()),
			None => {
				String::from("No image, to the moment. Probably because target folders are empty.")
			}
		}
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn get_current_pos(&self) -> Option<usize> {
		self.position
	}
}
