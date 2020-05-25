pub fn run(
	instructions: crate::cli_parsing::CliInstructions,
	user_data: std::sync::Arc<std::sync::Mutex<crate::user_data::UserData>>,
) {
	let port = { user_data.lock().unwrap().internal_server_port };

	if instructions.debug {
		println!(
			"DEBUG: internal web server will be launched at http://127.0.0.1:{}/",
			&port
		);
	}

	std::thread::spawn(move || {
		iron::Iron::new(move |req: &mut iron::prelude::Request| {
			let current = &user_data.lock().unwrap().get_current();

			let path_requested = format!("{}", req.url);
			let mut token = format!("http://127.0.0.1:{}/", port);
			token += &user_data.lock().unwrap().token;

			if path_requested == token {
				if instructions.debug {
					println!("DEBUG: receiving request to {}", &path_requested);
				}

				let file = std::fs::read(&current);

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
							eprintln!("ERROR: can not get file {} because : {}", current, e);
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
