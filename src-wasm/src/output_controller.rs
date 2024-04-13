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
