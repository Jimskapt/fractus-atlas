mod instructions_code;

pub fn run(
	instructions: crate::cli_parsing::CliInstructions,
	configuration: crate::configuration::Configuration,
	user_data: std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>,
	logger: charlie_buffalo::ConcurrentLogger,
) {
	let window_title = format!("{} V{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

	let html = include_str!("dist/main.html")
		.replace(
			r#"<script src="main.js"></script>"#,
			&format!("<script>{}</script>", include_str!("dist/main.js")),
		)
		.replace(
			r#"<script src="error_image.js"></script>"#,
			&format!("<script>{}</script>", include_str!("dist/error_image.js")),
		)
		.replace(
			r#"<script src="app.js"></script>"#,
			&format!("<script>{}</script>", include_str!("dist/app.js")),
		)
		.replace(
			r#"<script src="toasts.js"></script>"#,
			&format!("<script>{}</script>", include_str!("dist/toasts.js")),
		)
		.replace(
			r#"<link rel="stylesheet" href="main.css">"#,
			&format!(
				"<style type=\"text/css\">{}\n{}</style>",
				include_str!("dist/main.css"),
				configuration.custom_css
			),
		);

	let arc_for_window_data = std::sync::Arc::clone(&user_data);
	let instructions_for_window = instructions.clone();

	let main_window = web_view::builder()
		.title(&window_title)
		.content(web_view::Content::Html(&html))
		.size(800, 600)
		.resizable(true)
		.debug(instructions_for_window.debug)
		.user_data(arc_for_window_data)
		.invoke_handler(|webview, arg| {

			let simple_instruction = serde_json::from_str::<instructions_code::Instruction>(&arg).unwrap();
			let simple_instruction = serde_json::to_value(simple_instruction).unwrap();
			let simple_instruction = format!("{}", simple_instruction["instruction"]);
			let simple_instruction = simple_instruction.replace("\"", "");

			charlie_buffalo::push(&logger,
				vec![
					crate::LogLevel::DEBUG.into(),
					charlie_buffalo::Attr::new("component", "webview").into(),
					charlie_buffalo::Attr::new("event", simple_instruction).into(),
				],
				None
			);

			match serde_json::from_str(&arg).unwrap() {
				instructions_code::Instruction::Previous => {
					instructions_code::previous(webview, logger.clone());
				}
				instructions_code::Instruction::Next => {
					instructions_code::next(webview, logger.clone());
				}
				instructions_code::Instruction::Random => {
					instructions_code::random(webview, logger.clone());
				}
				instructions_code::Instruction::SetPosition { value } => {
					instructions_code::set_position(webview, logger.clone(), value);
				}
				instructions_code::Instruction::DoMove { into, toggle_popup } => {
					instructions_code::do_move(webview, logger.clone(), instructions_for_window.working_folder.clone(), into, toggle_popup);
				}
				instructions_code::Instruction::ShowBrowseTarget { id } => {
					instructions_code::show_browse_target(webview, logger.clone(), id);
				}
				instructions_code::Instruction::BrowseTargetFolders {
					folders,
					toggle_window,
				} => {

					let file_regex = match regex::RegexBuilder::new(&instructions_for_window.filter)
						.case_insensitive(true)
						.build()
					{
						Ok(res) => res,
						Err(e) => {
							let default_regex = crate::cli_parsing::CliInstructions::default().filter;
							charlie_buffalo::push(&logger,
								vec![
									crate::LogLevel::INFO.into(),
									charlie_buffalo::Attr::new("component", "webview").into(),
									charlie_buffalo::Attr::new("event", arg).into(),
								],
								Some(&format!("compilation of filter regex {} has failed (falling back to default {}) because : {}", &instructions_for_window.filter, &default_regex, e)),
							);

							regex::RegexBuilder::new(&default_regex)
								.case_insensitive(true)
								.build()
								.unwrap()
						}
					};

					instructions_code::browse_target_folders(
						webview,
						logger.clone(),
						&file_regex,
						instructions_for_window.sort.clone(),
						folders,
						toggle_window,
					);
				}
			}

			return Ok(());
		})
		.build()
		.unwrap_or_else(|e| panic!("Can not build main window : {}", e));

	let arc_for_dispatch = std::sync::Arc::clone(&user_data);
	let logger_for_window = logger.clone();

	main_window
		.handle()
		.dispatch(move |main_window| {
			let mut targets = arc_for_dispatch.lock().unwrap().targets.clone();
			let internal_server_port = arc_for_dispatch.lock().unwrap().internal_server_port;

			// ****** TARGETS ******

			targets.sort();
			let mut targets_buffer = String::from("[");
			targets_buffer += &targets
				.iter()
				.map(|target| format!("{}", web_view::escape(&target)))
				.collect::<Vec<String>>()
				.join(",");
			targets_buffer += "]";

			// ****** FOLDERS ******

			let mut folders: Vec<String> = vec![];
			if let Ok(childs) = std::fs::read_dir(&instructions.working_folder) {
				for entry in childs {
					if let Ok(entry) = entry {
						let path = entry.path();
						if path.is_dir() {
							folders.push(String::from(
								path.file_name()
									.unwrap_or_default()
									.to_str()
									.unwrap_or_default(),
							));
						}
					}
				}
			}

			let mut folders_buffer = String::from("[");
			folders_buffer += &folders
				.iter()
				.map(|target| format!("{}", web_view::escape(&target)))
				.collect::<Vec<String>>()
				.join(",");
			folders_buffer += "]";

			// ****** sending ******

			let js_instructions: String = format!(
				"STANDALONE_MODE=false;

App.data.debug = {};
App.data.internal_server_port = {};

App.remote.receive.set_targets({});
App.remote.receive.set_move_folders({});

App.methods.do_open(false);",
				if instructions.debug { "true" } else { "false" },
				internal_server_port,
				&targets_buffer,
				&folders_buffer
			);

			run_js(main_window, &js_instructions, logger_for_window)
		})
		.unwrap();

	charlie_buffalo::push(
		&logger,
		vec![
			crate::LogLevel::DEBUG.into(),
			charlie_buffalo::Attr::new("component", "webview").into(),
		],
		Some("attempting to display web_view window"),
	);

	main_window.run().unwrap();
}

pub fn run_js(
	webview: &mut web_view::WebView<std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>>,
	js_instructions: &str,
	logger: charlie_buffalo::ConcurrentLogger,
) -> web_view::WVResult {
	if !js_instructions.is_empty() {
		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "webview").into(),
				charlie_buffalo::Attr::new("event", "send_js").into(),
			],
			Some(&format!("sending\n```js\n{}\n```", &js_instructions)),
		);
		webview.eval(&js_instructions)
	} else {
		Ok(())
	}
}
