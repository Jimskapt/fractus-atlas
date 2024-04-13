mod utils;

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
			generate_input_folder(
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
		generate_input_folder(
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
			generate_output_folder(
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
		generate_output_folder(&document, &div, id, "", "", &[]);

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

fn generate_input_folder(
	document: &web_sys::Document,
	div: &web_sys::Element,
	id: usize,
	path: impl Into<String>,
	name: Option<impl Into<String>>,
	filters: &[common::FileFilter],
) {
	let fieldset = document.create_element("fieldset").unwrap();

	let path_string = path.into();
	let name_string = match name {
		Some(named) => named.into(),
		None => String::new(),
	};

	///////////////////

	let fieldset_legend = document.create_element("legend").unwrap();

	let name_input = document.create_element("input").unwrap();
	name_input.set_attribute("type", "text").unwrap();
	name_input
		.set_attribute("id", &format!("input_folder_name_{id}"))
		.unwrap();
	name_input
		.set_attribute("placeholder", "name of this input (optional)")
		.unwrap();
	name_input.set_attribute("value", &name_string).unwrap();

	fieldset_legend.append_child(&name_input).unwrap();
	fieldset.append_child(&fieldset_legend).unwrap();

	///////////////////

	let label = document.create_element("label").unwrap();
	label
		.set_attribute("for", &format!("input_folder_path_{id}"))
		.unwrap();
	label.set_inner_html("📂 path");

	fieldset.append_child(&label).unwrap();

	///////////////////

	let path_input = document.create_element("input").unwrap();
	path_input.set_attribute("type", "text").unwrap();
	path_input.set_attribute("class", "solo").unwrap();
	path_input
		.set_attribute("id", &format!("input_folder_path_{id}"))
		.unwrap();
	path_input.set_attribute("placeholder", "path").unwrap();
	path_input
		.set_attribute("onblur", "set_modified(false)")
		.unwrap();
	path_input.set_attribute("value", &path_string).unwrap();

	let command =
		Closure::wrap(Box::new(move || update_input_folders(id)) as Box<dyn FnMut() + 'static>);

	path_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	fieldset.append_child(&path_input).unwrap();

	///////////////////

	let filters_container = document.create_element("details").unwrap();
	filters_container
		.set_attribute("id", &format!("filters_{id}"))
		.unwrap();

	let filters_container_name = document.create_element("summary").unwrap();
	filters_container_name.set_inner_html("🔬 file filters");
	filters_container
		.append_child(&filters_container_name)
		.unwrap();

	let mut i_count = 0;
	for (i, filter) in filters.iter().enumerate() {
		generate_input_filter(
			document,
			&filters_container,
			id,
			i + 1,
			Some(filter.clone()),
		);
		i_count = i + 1;
	}

	i_count += 1;
	generate_input_filter(document, &filters_container, id, i_count, None);

	fieldset.append_child(&filters_container).unwrap();

	///////////////////

	div.append_child(&fieldset).unwrap();
}

fn generate_input_filter(
	document: &web_sys::Document,
	filters_container: &web_sys::Element,
	id: usize,
	i: usize,
	filter: Option<common::FileFilter>,
) {
	if i > 1 {
		let or = document.create_element("p").unwrap();
		or.set_attribute("class", "or_separator").unwrap();
		or.set_inner_html("OR");
		filters_container.append_child(&or).unwrap();
	}

	let (selector_value, input_value) = match filter {
		Some(common::FileFilter::Extension(file_name)) => {
			(String::from("extension"), file_name.clone())
		}
		Some(common::FileFilter::All) => todo!(),
		Some(common::FileFilter::MimeType(_)) => todo!(),
		Some(common::FileFilter::BaseFileName(_)) => todo!(),
		None => (String::from("extension"), String::new()),
	};

	let select = document.create_element("select").unwrap();
	select
		.set_attribute("id", &format!("input_folder_filter_type_{id}_{i}"))
		.unwrap();

	let select_extension = document.create_element("option").unwrap();
	select_extension
		.set_attribute("value", "extension")
		.unwrap();
	if selector_value == "extension" {
		select_extension
			.set_attribute("selected", "selected")
			.unwrap();
	}
	select_extension.set_inner_html("Extension");
	select.append_child(&select_extension).unwrap();

	filters_container.append_child(&select).unwrap();

	let filters_input = document.create_element("input").unwrap();
	filters_input.set_attribute("type", "text").unwrap();
	filters_input
		.set_attribute("id", &format!("input_folder_filter_value_{id}_{i}"))
		.unwrap();
	filters_input
		.set_attribute("placeholder", "filter")
		.unwrap();
	filters_input
		.set_attribute("onblur", "set_modified(false)")
		.unwrap();
	filters_input.set_attribute("value", &input_value).unwrap();

	let command =
		Closure::wrap(Box::new(move || update_input_filters(id, i)) as Box<dyn FnMut() + 'static>);

	filters_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	filters_container.append_child(&filters_input).unwrap();
}

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
		generate_input_folder(
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
		generate_input_filter(&document, &filters_container, id, i + 1, None);
	}
}

fn generate_output_folder(
	document: &web_sys::Document,
	div: &web_sys::Element,
	id: usize,
	path: impl Into<String>,
	name: impl Into<String>,
	shortcuts: &[common::Shortcut],
) {
	let fieldset = document.create_element("fieldset").unwrap();

	let path_string = path.into();
	let name_string = name.into();

	///////////////////

	let fieldset_legend = document.create_element("legend").unwrap();

	let name_output = document.create_element("input").unwrap();
	name_output.set_attribute("type", "text").unwrap();
	name_output
		.set_attribute("id", &format!("output_folder_name_{id}"))
		.unwrap();
	name_output
		.set_attribute("placeholder", "name of this output")
		.unwrap();
	name_output.set_attribute("value", &name_string).unwrap();

	fieldset_legend.append_child(&name_output).unwrap();
	fieldset.append_child(&fieldset_legend).unwrap();

	///////////////////

	let label = document.create_element("label").unwrap();
	label
		.set_attribute("for", &format!("output_folder_path_{id}"))
		.unwrap();
	label.set_inner_html("🚚 path");

	fieldset.append_child(&label).unwrap();

	///////////////////

	let path_output = document.create_element("input").unwrap();
	path_output.set_attribute("type", "text").unwrap();
	path_output.set_attribute("class", "solo").unwrap();
	path_output
		.set_attribute("id", &format!("output_folder_path_{id}"))
		.unwrap();
	path_output.set_attribute("placeholder", "path").unwrap();
	path_output
		.set_attribute("onblur", "set_modified(false)")
		.unwrap();
	path_output.set_attribute("value", &path_string).unwrap();

	let command =
		Closure::wrap(Box::new(move || update_output_folders(id)) as Box<dyn FnMut() + 'static>);

	path_output
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	fieldset.append_child(&path_output).unwrap();

	///////////////////

	let shortcuts_container = document.create_element("details").unwrap();
	shortcuts_container.set_attribute("open", "open").unwrap();
	shortcuts_container
		.set_attribute("id", &format!("shortcuts_{id}"))
		.unwrap();

	let shortcuts_container_name = document.create_element("summary").unwrap();
	shortcuts_container_name.set_inner_html("⌨️ (optional) shortcut(s)");
	shortcuts_container
		.append_child(&shortcuts_container_name)
		.unwrap();

	let mut i_count = 0;
	for (i, shortcut) in shortcuts.iter().enumerate() {
		generate_output_shortcut(
			document,
			&shortcuts_container,
			id,
			i + 1,
			Some(shortcut.clone()),
		);
		i_count = i + 1;
	}

	i_count += 1;
	generate_output_shortcut(document, &shortcuts_container, id, i_count, None);

	fieldset.append_child(&shortcuts_container).unwrap();

	///////////////////

	div.append_child(&fieldset).unwrap();
}

fn generate_output_shortcut(
	document: &web_sys::Document,
	shortcuts_container: &web_sys::Element,
	id: usize,
	i: usize,
	shortcut: Option<common::Shortcut>,
) {
	if i > 1 {
		let or = document.create_element("p").unwrap();
		or.set_attribute("class", "or_separator").unwrap();
		or.set_inner_html("OR");
		shortcuts_container.append_child(&or).unwrap();
	}

	let (selector_value, output_value) = match shortcut {
		Some(common::Shortcut::Key(key)) => (String::from("key"), key.clone()),
		None => (String::from("key"), String::new()),
	};

	let select = document.create_element("select").unwrap();
	select
		.set_attribute("id", &format!("output_folder_shortcut_type_{id}_{i}"))
		.unwrap();

	let select_key = document.create_element("option").unwrap();
	select_key.set_attribute("value", "key").unwrap();
	if selector_value == "key" {
		select_key.set_attribute("selected", "selected").unwrap();
	}
	select_key.set_inner_html("Key");
	select.append_child(&select_key).unwrap();

	shortcuts_container.append_child(&select).unwrap();

	let shortcut_input = document.create_element("input").unwrap();
	shortcut_input.set_attribute("type", "text").unwrap();
	shortcut_input
		.set_attribute("id", &format!("output_folder_shortcut_value_{id}_{i}"))
		.unwrap();
	shortcut_input
		.set_attribute("placeholder", "shortcut")
		.unwrap();
	shortcut_input
		.set_attribute("onblur", "set_modified(false)")
		.unwrap();
	shortcut_input
		.set_attribute("value", &output_value)
		.unwrap();

	let command = Closure::wrap(
		Box::new(move || update_output_shortcuts(id, i)) as Box<dyn FnMut() + 'static>
	);

	shortcut_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	shortcuts_container.append_child(&shortcut_input).unwrap();
}

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
		generate_output_folder(&document, &div, id + 1, "", "", &[]);
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
		generate_output_shortcut(&document, &shortcuts_container, id, i + 1, None);
	}
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

	let input_folders = {
		let mut i = 0;
		let mut input_folders = vec![];
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
							filters
								.push(common::FileFilter::Extension(value.trim().to_lowercase()));
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

		input_folders
	};

	//////////////////////

	let output_folders = {
		let mut i = 0;
		let mut output_folders = vec![];
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
							.query_selector(&format!(
								"#output_folder_shortcut_type_{i}_{i_shortcut}"
							))
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

		output_folders
	};

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
