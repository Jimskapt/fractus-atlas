use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn update_output_folders(id: usize) {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let div = document
		.query_selector("#output_folders_fields")
		.unwrap()
		.unwrap();

	let output_folder_path_value = document
		.query_selector(&format!("#output_folder_path_{id}"))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();

	if !output_folder_path_value.trim().is_empty()
		&& document
			.query_selector(&format!("#output_folder_path_{}", id + 1))
			.unwrap()
			.is_none()
	{
		crate::output_view::generate_output_folder(&document, &div, id + 1, "", "", &[]);
	}
}

#[wasm_bindgen]
pub fn update_output_shortcuts(id: usize, i: usize) {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let shortcuts_container = document
		.query_selector(&format!("#shortcuts_{id}"))
		.unwrap()
		.unwrap();

	let output_value = document
		.query_selector(&format!("#output_folder_shortcut_value_{id}_{i}"))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();

	if !output_value.trim().is_empty()
		&& document
			.query_selector(&format!("#output_folder_shortcut_value_{id}_{}", i + 1))
			.unwrap()
			.is_none()
	{
		crate::output_view::generate_output_shortcut(
			&document,
			&shortcuts_container,
			id,
			i + 1,
			None,
		);
	}
}

pub fn save() -> (Vec<common::OutputFolder>, Vec<common::SaveMessage>) {
	let mut output_folders = vec![];
	let mut messages = vec![];

	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let mut i = 0;
	while let Some(field) = document
		.query_selector(&format!("#output_folder_path_{i}"))
		.unwrap()
	{
		let path_value = field
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.value();

		let name_value = document
			.query_selector(&format!("#output_folder_name_{i}"))
			.unwrap()
			.unwrap()
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.value();

		if !path_value.trim().is_empty() {
			let mut shortcuts = vec![];
			let mut i_shortcut = 1;
			while let Some(shortcut_field) = document
				.query_selector(&format!("#output_folder_shortcut_value_{i}_{i_shortcut}"))
				.unwrap()
			{
				let value = shortcut_field
					.dyn_into::<web_sys::HtmlInputElement>()
					.unwrap()
					.value();

				if !value.trim().is_empty() {
					let shortcut_type = document
						.query_selector(&format!("#output_folder_shortcut_type_{i}_{i_shortcut}"))
						.unwrap()
						.unwrap()
						.dyn_into::<web_sys::HtmlSelectElement>()
						.unwrap()
						.value();

					if shortcut_type == "key" {
						shortcuts.push(common::Shortcut::Key(value.trim().to_lowercase()));
					} else {
						todo!()
					}
				}

				i_shortcut += 1;
			}

			output_folders.push(common::OutputFolder {
				name: String::from(name_value.trim()),
				path: std::path::PathBuf::from(path_value),
				shortcuts_or: shortcuts,
			});
		}

		i += 1;
	}

	(output_folders, messages)
}
