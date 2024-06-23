use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Settings {
	pub input_folders: Vec<InputFolder>,
	exclude_paths: Vec<InputFolder>,
	pub output_folders: Vec<OutputFolder>,
	pub steps_after_move: isize,
	pub sorting: SortingOrder,
	pub confirm_rename: Option<bool>,
	pub settings_version: Option<String>,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			input_folders: vec![InputFolder {
				path: PathBuf::from("."),
				name: Some(String::from("execution folder")),
				filters: vec![
					FileFilter::Extension(String::from("png")),
					FileFilter::Extension(String::from("jpg")),
					FileFilter::Extension(String::from("jpeg")),
					FileFilter::Extension(String::from("gif")),
					FileFilter::Extension(String::from("webp")),
					FileFilter::Extension(String::from("bmp")),
					FileFilter::Extension(String::from("ico")),
					FileFilter::Extension(String::from("tif")),
					FileFilter::Extension(String::from("tiff")),
				],
				recursivity: Some(false),
			}],
			exclude_paths: vec![],
			output_folders: vec![OutputFolder {
				path: PathBuf::from("fractus-atlas"),
				name: String::from("fractus-atlas"),
				shortcuts_or: vec![Shortcut::Key(String::from("m"))],
			}],
			steps_after_move: 1,
			sorting: SortingOrder::FileName,
			confirm_rename: Some(true),
			settings_version: Some(String::from(env!("CARGO_PKG_VERSION"))),
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InputFolder {
	pub path: PathBuf,
	pub name: Option<String>,
	pub filters: Vec<FileFilter>,
	pub recursivity: Option<bool>,
}
impl InputFolder {
	pub fn filter(&self, path: &std::path::Path, settings_path: Option<PathBuf>) -> bool {
		log::trace!("path = {path:?}");
		log::trace!("filters = {:?}", self.filters);

		let res = path.starts_with(crate::build_absolute_path(&self.path, settings_path))
			&& self.filters.iter().any(|filter| filter.filter(path));

		log::trace!("res = {res}");

		res
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum SortingOrder {
	FileName,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OutputFolder {
	pub path: PathBuf,
	pub name: String,
	pub shortcuts_or: Vec<Shortcut>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum FileFilter {
	All,
	Extension(String),
	MimeType(String),
	BaseFileName(String),
}
impl FileFilter {
	fn filter(&self, path: &std::path::Path) -> bool {
		match self {
			Self::All => true,
			Self::Extension(expected_extension) => {
				if let Some(file_extension) = path.extension() {
					file_extension.to_string_lossy().to_lowercase().trim()
						== expected_extension.to_lowercase().trim()
				} else {
					false
				}
			}
			Self::MimeType(expected_type) => todo!(),
			Self::BaseFileName(expected_base_filename) => {
				if let Some(file_name) = path.file_name() {
					file_name
						.to_string_lossy()
						.to_lowercase()
						.trim()
						.starts_with(expected_base_filename)
				} else {
					false
				}
			}
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum Shortcut {
	Key(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum SaveMessage {
	Error(String),
	Warning(String),
	Confirm(String),
}

pub fn build_absolute_path(
	input: impl Into<std::path::PathBuf>,
	settings_path: Option<std::path::PathBuf>,
) -> std::path::PathBuf {
	let input_paths = input.into();

	if input_paths.is_absolute() {
		input_paths.clone()
	} else if let Some(some_settings_path) = &settings_path {
		some_settings_path.parent().unwrap().join(&input_paths)
	} else if let Ok(current_dir) = std::env::current_dir() {
		current_dir.join(&input_paths)
	} else if let Some(exec) = std::env::args().next() {
		std::path::PathBuf::from(exec)
			.parent()
			.unwrap()
			.join(&input_paths)
	} else {
		panic!();
	}
}
