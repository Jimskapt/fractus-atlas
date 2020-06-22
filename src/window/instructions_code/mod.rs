mod browse_target_folders;
mod do_move;
mod next;
mod open_current;
mod previous;
mod random;
mod set_position;
mod show_browse_target;

pub use browse_target_folders::browse_target_folders;
pub use do_move::do_move;
pub use next::next;
pub use open_current::open_current_file;
pub use open_current::open_current_folder;
pub use previous::previous;
pub use random::random;
pub use set_position::set_position;
pub use show_browse_target::show_browse_target;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
#[serde(tag = "instruction", rename_all = "PascalCase")]
pub enum Instruction {
	Previous,
	Next,
	Random,
	OpenCurrentFile,
	OpenCurrentFolder,
	SetPosition {
		value: usize,
	},
	DoMove {
		into: String,
		toggle_popup: bool,
	},
	ShowBrowseTarget {
		id: usize
	},
	BrowseTargetFolders {
		folders: Vec<String>,
		sort_order: String,
		toggle_window: bool,
	},
}
