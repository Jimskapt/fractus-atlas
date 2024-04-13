use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn update_input_folders(id: usize) {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let div = document
		.query_selector("#input_folders_fields")
		.unwrap()
		.unwrap();

	let input_folder_path_value = document
		.query_selector(&format!("input#input_folder_path_{id}"))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();

	if !input_folder_path_value.trim().is_empty()
		&& document
			.query_selector(&format!("#input_folder_path_{}", id + 1))
			.unwrap()
			.is_none()
	{
		crate::input_view::generate_input_folder(
			&document,
			&div,
			id + 1,
			"",
			Some(""),
			&common::Settings::default()
				.input_folders
				.first()
				.unwrap()
				.filters,
		);
	}
}

#[wasm_bindgen]
pub fn update_input_filters(id: usize, i: usize) {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let filters_container = document
		.query_selector(&format!("#filters_{id}"))
		.unwrap()
		.unwrap();

	let input_value = document
		.query_selector(&format!("#input_folder_filter_value_{id}_{i}"))
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();

	if !input_value.trim().is_empty()
		&& document
			.query_selector(&format!("#input_folder_filter_value_{id}_{}", i + 1))
			.unwrap()
			.is_none()
	{
		crate::input_view::generate_input_filter(&document, &filters_container, id, i + 1, None);
	}
}

pub fn save() -> (Vec<common::InputFolder>, Vec<common::SaveMessage>) {
	let mut input_folders = vec![];
	let mut messages = vec![];

	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	let mut i = 0;
	while let Some(field) = document
		.query_selector(&format!("#input_folder_path_{i}"))
		.unwrap()
	{
		let path_value = field
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.value();

		let name_value = document
			.query_selector(&format!("#input_folder_name_{i}"))
			.unwrap()
			.unwrap()
			.dyn_into::<web_sys::HtmlInputElement>()
			.unwrap()
			.value();

		if !path_value.trim().is_empty() {
			let mut filters = vec![];
			let mut i_filter = 1;
			while let Some(filter_field) = document
				.query_selector(&format!("#input_folder_filter_value_{i}_{i_filter}"))
				.unwrap()
			{
				let value = filter_field
					.dyn_into::<web_sys::HtmlInputElement>()
					.unwrap()
					.value();

				if !value.trim().is_empty() {
					let filter_type = document
						.query_selector(&format!("#input_folder_filter_type_{i}_{i_filter}"))
						.unwrap()
						.unwrap()
						.dyn_into::<web_sys::HtmlSelectElement>()
						.unwrap()
						.value();

					if filter_type == "extension" {
						filters.push(common::FileFilter::Extension(value.trim().to_lowercase()));
					} else {
						todo!()
					}
				}

				i_filter += 1;
			}

			input_folders.push(common::InputFolder {
				name: if name_value.trim().is_empty() {
					None
				} else {
					Some(String::from(name_value.trim()))
				},
				path: std::path::PathBuf::from(path_value),
				filters,
			});
		}

		i += 1;
	}

	(input_folders, messages)
}
