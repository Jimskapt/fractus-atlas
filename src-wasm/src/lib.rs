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

	let settings_path: Option<std::path::PathBuf> =
		serde_wasm_bindgen::from_value(invoke("get_settings_path", JsValue::NULL).await).unwrap();
	document
		.query_selector("input#settings_path")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.set_value(&match settings_path {
			Some(path) => format!("{}", path.display()),
			None => String::new(),
		});

	//////////////////////

	let settings: common::Settings =
		serde_wasm_bindgen::from_value(invoke("get_settings", JsValue::NULL).await).unwrap();

	let dyn_settings_form = document
		.query_selector("#dyn_settings_form")
		.unwrap()
		.unwrap();

	while let Some(child) = dyn_settings_form.first_child() {
		dyn_settings_form.remove_child(&child).unwrap();
	}

	//////////////////////

	{
		let p = document.create_element("p").unwrap();

		let backend_version: String =
			serde_wasm_bindgen::from_value(invoke("get_backend_version", JsValue::NULL).await)
				.unwrap();

		p.set_inner_html(&format!(
			r#"
				Frontend version : {front}<br>
				Backend version : {backend_version}<br>
				Settings version : {sett}<br>
				Website : <a href="{website}" target="_blank">{website}</a>"#,
			front = env!("CARGO_PKG_VERSION"),
			website = env!("CARGO_PKG_REPOSITORY"),
			sett = settings
				.settings_version
				.unwrap_or_else(|| { String::from("2.0.0") })
		));

		dyn_settings_form.append_child(&p).unwrap();
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

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("class", "solo").unwrap();

		let input = document.create_element("input").unwrap();
		input.set_attribute("type", "checkbox").unwrap();
		input.set_attribute("id", "confirm_rename").unwrap();
		input
			.set_attribute("onchange", "set_modified(false)")
			.unwrap();
		if settings.confirm_rename.unwrap_or(true) {
			input.set_attribute("checked", "checked").unwrap();
		}

		label.set_inner_html("&nbsp;confirm renaming in path field");
		label
			.insert_before(&input, label.first_child().as_ref())
			.unwrap();

		dyn_settings_form.append_child(&label).unwrap();
	}

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("class", "solo").unwrap();

		let input = document.create_element("input").unwrap();
		input.set_attribute("type", "checkbox").unwrap();
		input.set_attribute("id", "move_to_newest").unwrap();
		input
			.set_attribute("onchange", "set_modified(false)")
			.unwrap();
		if settings.move_to_newest.unwrap_or(false) {
			input.set_attribute("checked", "checked").unwrap();
		}

		label.set_inner_html(
			"&nbsp;if on last image, move on to the newest which will appear later",
		);
		label
			.insert_before(&input, label.first_child().as_ref())
			.unwrap();

		dyn_settings_form.append_child(&label).unwrap();
	}

	//////////////////////

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
				input_folder.recursivity.unwrap_or(false),
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
			false,
			&common::Settings::default()
				.input_folders
				.first()
				.unwrap()
				.filters,
		);

		dyn_settings_form.append_child(&div).unwrap();
	}

	//////////////////////

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
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSettingsPayload {
	new_settings: common::Settings,
}

#[wasm_bindgen]
pub async fn save_settings() {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();

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
	let confirm_rename = document
		.query_selector("#confirm_rename")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.checked();
	let move_to_newest = document
		.query_selector("#move_to_newest")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.checked();

	//////////////////////

	let mut new_settings: common::Settings =
		serde_wasm_bindgen::from_value(invoke("get_settings", JsValue::NULL).await).unwrap();
	new_settings.steps_after_move = steps_after_move;
	new_settings.input_folders = input_folders;
	new_settings.output_folders = output_folders;
	new_settings.confirm_rename = Some(confirm_rename);
	new_settings.move_to_newest = Some(move_to_newest);

	//////////////////////

	let messages: Vec<common::SaveMessage> = serde_wasm_bindgen::from_value(
		invoke(
			"set_settings",
			serde_wasm_bindgen::to_value(&SaveSettingsPayload { new_settings }).unwrap(),
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
		build_settings_form().await;
	}
}
