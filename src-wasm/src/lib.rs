#![allow(clippy::needless_return)]
#![deny(clippy::shadow_reuse)]
#![deny(clippy::shadow_same)]
#![deny(clippy::shadow_unrelated)]
#![deny(clippy::unwrap_in_result)]

mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
	utils::set_panic_hook();

	/*
	let name_input = unsafe { document.create_element("input").unwrap() };
	name_input.set_attribute("type", "text").unwrap();
	name_input.set_attribute("id", "name-input").unwrap();

	let result_paragraph = unsafe { document.create_element("p").unwrap() };
	result_paragraph.set_inner_html("Result:");

	let request = format!("{{\"name\": \"John Doe\"}}");
	let command =
		Closure::wrap(Box::new(move || invoke("test").await) as Box<dyn FnMut() + 'static>);

	/*
	name_input
		.add_event_listener_with_callback("input", move |_| command.emit())
		.unwrap();
	*/
	command.forget();

	body.append_child(&name_input).unwrap();
	body.append_child(&result_paragraph).unwrap();

	// greet();
	*/

	Ok(())
}

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
	async fn invoke(name: &str, payload: JsValue) -> JsValue;
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

	/*
	let pre = document.create_element("pre").unwrap();
	pre.set_text_content(Some(&serde_json::to_string_pretty(&settings).unwrap()));
	pre.set_attribute("style", "font-size:10px;line-height:normal;").unwrap();
	*/

	let dyn_settings_form = document
		.query_selector("#dyn_settings_form")
		.unwrap()
		.unwrap();

	{
		let label = document.create_element("label").unwrap();
		label.set_attribute("for", "steps_after_move").unwrap();
		label.set_inner_html("🦘 Steps after move");

		dyn_settings_form.append_child(&label).unwrap();

		let input = document.create_element("input").unwrap();
		input.set_attribute("type", "number").unwrap();
		input.set_attribute("id", "steps_after_move").unwrap();
		input.set_attribute("onchange", "set_modified()").unwrap();
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

	let settings_path_value = document
		.query_selector("input#settings_path")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();
	let settings_path = settings_path_value.parse().unwrap();

	let steps_after_move_value = document
		.query_selector("input#steps_after_move")
		.unwrap()
		.unwrap()
		.dyn_into::<web_sys::HtmlInputElement>()
		.unwrap()
		.value();
	let steps_after_move = steps_after_move_value.parse().unwrap();

	let mut new_settings = common::Settings::default();
	new_settings.steps_after_move = steps_after_move;

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

	for message in messages {
		let p = document.create_element("p").unwrap();
		match message {
			common::SaveMessage::Error(msg) => {
				p.set_attribute("class", "error").unwrap();
				p.set_text_content(Some(&format!("❌ {msg}")));
			}
			common::SaveMessage::Warning(msg) => {
				p.set_attribute("class", "warning").unwrap();
				p.set_text_content(Some(&format!("⚠️ {msg}")));
			}
			common::SaveMessage::Confirm(msg) => {
				p.set_attribute("class", "confirm").unwrap();
				p.set_text_content(Some(&format!("✔️ {msg}")));
			}
		};
		save_messages_container.append_child(&p).unwrap();
	}
}
