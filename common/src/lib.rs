#![allow(clippy::needless_return)]
#![deny(clippy::shadow_reuse)]
#![deny(clippy::shadow_same)]
#![deny(clippy::shadow_unrelated)]
#![deny(clippy::unwrap_in_result)]

use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Settings {
	pub input_folders: Vec<InputFolder>,
	exclude_paths: Vec<InputFolder>,
	pub output_folders: Vec<OutputFolder>,
	pub steps_after_move: isize,
	sorting: SortingOrder,
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
			}],
			exclude_paths: vec![],
			output_folders: vec![OutputFolder {
				path: PathBuf::from("fractus-atlas"),
				name: String::from("fractus-atlas"),
				shortcuts_or: vec![Shortcut::Key(String::from("m"))],
			}],
			steps_after_move: 1,
			sorting: SortingOrder::FileName,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InputFolder {
	pub path: PathBuf,
	name: Option<String>,
	filters: Vec<FileFilter>,
}
impl InputFolder {
	pub fn filter(&self, path: &std::path::Path) -> bool {
		self.filters.iter().any(|filter| filter.filter(path))
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
enum SortingOrder {
	FileName,
	Modified,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OutputFolder {
	pub path: PathBuf,
	pub name: String,
	pub shortcuts_or: Vec<Shortcut>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
enum FileFilter {
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