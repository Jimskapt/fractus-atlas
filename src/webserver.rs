pub fn run(
	instructions: crate::cli_parsing::CliInstructions,
	user_data: std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>,
	logger: charlie_buffalo::ConcurrentLogger,
) {
	let port = { user_data.lock().unwrap().internal_server_port };
	charlie_buffalo::push(
		&logger,
		vec![
			crate::LogLevel::DEBUG.into(),
			charlie_buffalo::Attr::new("stage", "webserver").into(),
			charlie_buffalo::Attr::new("port", &port).into(),
		],
		Some(&format!(
			"internal web server will be launched at http://127.0.0.1:{}/",
			&port
		)),
	);

	std::thread::spawn(move || {
		iron::Iron::new(move |req: &mut iron::prelude::Request| {
			if !user_data.lock().unwrap().images.is_empty() {
				let path_requested = format!("{}", req.url);
				let allowed: Vec<crate::user_data::Image> = {
					let local_user_data = user_data.lock().unwrap();

					let mut result = vec![];
					if let Some(image) = local_user_data.get_active() {
						result.push(image);
					}
					if let Some(image) = local_user_data.get_previous() {
						result.push(image);
					}
					if let Some(image) = local_user_data.get_next() {
						result.push(image);
					}

					result
				};

				let search = allowed
					.into_iter()
					.filter(|i| path_requested == format!("http://127.0.0.1:{}/{}", port, i.token))
					.collect::<Vec<crate::user_data::Image>>();

				if !search.is_empty() {
					if instructions.debug {
						println!("DEBUG: receiving request to {}", &path_requested);
					}

					charlie_buffalo::push(
						&logger,
						vec![
							crate::LogLevel::DEBUG.into(),
							charlie_buffalo::Attr::new("component", "webserver").into(),
						],
						Some(&format!("receiving request to {}", &path_requested)),
					);

					let path = &search.first().unwrap().current;
					let file = std::fs::read(&path);

					match file {
						Ok(path) => {
							let mime = tree_magic::from_u8(&path);
							let mut res = iron::Response::with((iron::status::Ok, path));
							res.headers
								.set_raw("Content-Type", vec![mime.as_bytes().to_vec()]);
							Ok(res)
						}
						Err(e) => {
							if instructions.debug {
								eprintln!("ERROR: can not get file {:?} because : {}", path, e);
							}

							match e.kind() {
								std::io::ErrorKind::NotFound => Err(iron::IronError::new(
									StringError(String::from("404 : NOT FOUND")),
									iron::status::Status::NotFound,
								)),
								std::io::ErrorKind::PermissionDenied => Err(iron::IronError::new(
									StringError(String::from("401 : UNAUTHORIZED")),
									iron::status::Status::Unauthorized,
								)),
								_ => Err(iron::IronError::new(
									StringError(String::from("500 : INTERNAL ERROR")),
									iron::status::Status::InternalServerError,
								)),
							}
						}
					}
				} else {
					if instructions.debug {
						println!(
							"DEBUG: the token does not match with request {}",
							&path_requested
						);
					}

					Err(iron::IronError::new(
						StringError(String::from("403 : FORBIDDEN")),
						iron::status::Status::Forbidden,
					))
				}
			} else {
				Err(iron::IronError::new(
					StringError(String::from("404 : NOT FOUND")),
					iron::status::Status::NotFound,
				))
			}
		})
		.http(&format!("127.0.0.1:{}", &port))
		.unwrap();
	});
}

#[derive(Debug)]
struct StringError(String);
impl std::fmt::Display for StringError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Debug::fmt(self, f)
	}
}
impl std::error::Error for StringError {
	fn description(&self) -> &str {
		&*self.0
	}
}
