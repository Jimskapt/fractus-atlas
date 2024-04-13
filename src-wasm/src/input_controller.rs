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
