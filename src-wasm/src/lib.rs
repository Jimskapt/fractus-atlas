mod utils;

mod input_controller;
mod input_view;

mod output_controller;
mod output_view;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
	#[cfg(debug_assertions)]
	utils::set_panic_hook();

	Ok(())
}

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
	async fn invoke(name: &str, payload: JsValue) -> JsValue;
	fn set_unmodified();
}

#[wasm_bindgen]
pub async fn build_settings_form() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	//////////////////////

	let settings_path: String =
		serde_wasm_bindgen::from_value(invoke("get_settings_path", JsValue::NULL).await).unwrap();
	document
		.query_selector("input#settings_path")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.set_value(&settings_path);

	//////////////////////

	let settings: common::Settings =
		serde_wasm_bindgen::from_value(invoke("get_settings", JsValue::NULL).await).unwrap();

	let dyn_settings_form = document
		.query_selector("#dyn_settings_form")
		.unwrap()
		.unwrap();

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("class", "solo").unwrap();
		label.set_inner_html("📂 Input folders");

		dyn_settings_form.append_child(&label).unwrap();

		let div = document.create_element("div").unwrap();
		div.set_attribute("id", "input_folders_fields").unwrap();

		let mut id = 0;
		for (i, input_folder) in settings.input_folders.iter().enumerate() {
			input_view::generate_input_folder(
				&document,
				&div,
				i,
				format!("{}", input_folder.path.display()),
				input_folder.name.as_ref(),
				&input_folder.filters,
			);
			id = i;
		}

		id += 1;
		input_view::generate_input_folder(
			&document,
			&div,
			id,
			"",
			Some(""),
			&common::Settings::default()
				.input_folders
				.first()
				.unwrap()
				.filters,
		);

		dyn_settings_form.append_child(&div).unwrap();
	}

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("class", "solo").unwrap();
		label.set_inner_html("🚚 Output folders");

		dyn_settings_form.append_child(&label).unwrap();

		let div = document.create_element("div").unwrap();
		div.set_attribute("id", "output_folders_fields").unwrap();

		let mut id = 0;
		for (i, output_folder) in settings.output_folders.iter().enumerate() {
			output_view::generate_output_folder(
				&document,
				&div,
				i,
				format!("{}", output_folder.path.display()),
				&output_folder.name,
				&output_folder.shortcuts_or,
			);
			id = i;
		}

		id += 1;
		output_view::generate_output_folder(&document, &div, id, "", "", &[]);

		dyn_settings_form.append_child(&div).unwrap();
	}

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("class", "solo").unwrap();
		label.set_attribute("for", "steps_after_move").unwrap();
		label.set_inner_html("🦘 Steps after move");

		dyn_settings_form.append_child(&label).unwrap();

		let input = document.create_element("input").unwrap();
		input.set_attribute("class", "solo").unwrap();
		input.set_attribute("type", "number").unwrap();
		input.set_attribute("id", "steps_after_move").unwrap();
		input
			.set_attribute("onchange", "set_modified(false)")
			.unwrap();
		input
			.set_attribute("value", &format!("{}", settings.steps_after_move))
			.unwrap();

		dyn_settings_form.append_child(&input).unwrap();
	}

	// dyn_settings_form.append_child(&pre).unwrap();
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSettingsPayload {
	settings_path: std::path::PathBuf,
	new_settings: common::Settings,
}

#[wasm_bindgen]
pub async fn save_settings() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

	//////////////////////

	let settings_path_value = document
		.query_selector("input#settings_path")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();
	let settings_path = settings_path_value.parse().unwrap();

	//////////////////////

	let steps_after_move_value = document
		.query_selector("input#steps_after_move")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();
	let steps_after_move = steps_after_move_value.parse().unwrap();

	//////////////////////

	let (input_folders, input_messages) = input_controller::save();
	let (output_folders, output_messages) = output_controller::save();

	//////////////////////

	let mut new_settings = common::Settings::default();
	new_settings.steps_after_move = steps_after_move;
	new_settings.input_folders = input_folders;
	new_settings.output_folders = output_folders;

	//////////////////////

	let messages: Vec<common::SaveMessage> = serde_wasm_bindgen::from_value(
		invoke(
			"set_settings",
			serde_wasm_bindgen::to_value(&SaveSettingsPayload {
				settings_path,
				new_settings,
			})
			.unwrap(),
		)
		.await,
	)
	.unwrap();

	let save_messages_container = document.query_selector("#save_messages").unwrap().unwrap();
	while let Some(child) = save_messages_container.last_child() {
		save_messages_container.remove_child(&child).unwrap();
	}

	let mut is_ok = true;
	for message in messages {
		let p = document.create_element("p").unwrap();
		match message {
			common::SaveMessage::Error(msg) => {
				p.set_attribute("class", "error").unwrap();
				p.set_text_content(Some(&format!("❌ {msg}")));
				is_ok = false;
			}
			common::SaveMessage::Warning(msg) => {
				p.set_attribute("class", "warning").unwrap();
				p.set_text_content(Some(&format!("⚠️ {msg}")));
				is_ok = false;
			}
			common::SaveMessage::Confirm(msg) => {
				p.set_attribute("class", "confirm").unwrap();
				p.set_text_content(Some(&format!("✔️ {msg}")));
			}
		};
		save_messages_container.append_child(&p).unwrap();
	}

	if is_ok {
		set_unmodified();
	}
}
