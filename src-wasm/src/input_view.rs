use wasm_bindgen::prelude::*;

pub fn generate_input_folder(
	document: &web_sys::Document,
	div: &web_sys::Element,
	id: usize,
	path: impl Into<String>,
	name: Option<impl Into<String>>,
	recursivity: bool,
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

	let command = Closure::wrap(
		Box::new(move || crate::input_controller::update_input_folders(id))
			as Box<dyn FnMut() + 'static>,
	);

	path_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	fieldset.append_child(&path_input).unwrap();

	///////////////////

	let recursivity_check = document.create_element("input").unwrap();
	recursivity_check.set_attribute("type", "checkbox").unwrap();
	recursivity_check
		.set_attribute("id", &format!("folder_recursivity_{id}"))
		.unwrap();
	recursivity_check
		.set_attribute("onchange", "set_modified(false)")
		.unwrap();
	if recursivity {
		recursivity_check
			.set_attribute("checked", "checked")
			.unwrap();
	}

	fieldset.append_child(&recursivity_check).unwrap();

	let recursivity_label = document.create_element("label").unwrap();
	recursivity_label
		.set_attribute("for", &format!("folder_recursivity_{id}"))
		.unwrap();
	recursivity_label.set_text_content(Some(" is recursive : searching files in its sub-folders"));

	fieldset.append_child(&recursivity_label).unwrap();

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

pub fn generate_input_filter(
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
		Closure::wrap(
			Box::new(move || crate::input_controller::update_input_filters(id, i))
				as Box<dyn FnMut() + 'static>,
		);

	filters_input
		.add_event_listener_with_callback("blur", command.as_ref().unchecked_ref())
		.unwrap();
	command.forget();

	filters_container.append_child(&filters_input).unwrap();
}
