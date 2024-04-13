use wasm_bindgen::prelude::*;

pub fn generate_output_folder(
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
		Closure::wrap(
			Box::new(move || crate::output_controller::update_output_folders(id))
				as Box<dyn FnMut() + 'static>,
		);

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

pub fn generate_output_shortcut(
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

	let command =
		Closure::wrap(
			Box::new(move || crate::output_controller::update_output_shortcuts(id, i))
				as Box<dyn FnMut() + 'static>,
		);

	shortcut_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	shortcuts_container.append_child(&shortcut_input).unwrap();
}
