#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Configuration {
	pub custom_css: String,
}
impl Default for Configuration {
	fn default() -> Configuration {
		Configuration {
			custom_css: String::new(),
		}
	}
}
impl
	std::convert::From<(
		&crate::cli_parsing::CliInstructions,
		std::sync::Arc<std::sync::Mutex<charlie_buffalo::Logger>>,
	)> for Configuration
{
	fn from(
		input: (
			&crate::cli_parsing::CliInstructions,
			charlie_buffalo::ConcurrentLogger,
		),
	) -> Self {
		let (instructions, logger) = input;

		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			Some("attempting to load configuration"),
		);

		let mut result = Configuration::default();

		let configuration_path = std::path::Path::new(&instructions.configuration_path);

		if configuration_path.exists() {
			let configuration = std::fs::read_to_string(&instructions.configuration_path);

			match configuration {
				Ok(configuration) => match toml::from_str(&configuration) {
					Ok(configuration) => {
						result = configuration;
					}
					Err(e) => {
						charlie_buffalo::push(
							&logger,
							vec![
								crate::LogLevel::INFO.into(),
								charlie_buffalo::Attr::new("component", "app").into(),
								charlie_buffalo::Attr::new("stage", "configuration").into(),
							],
							Some(
								&format!("error while parsing configuration (this is not fatal, we falling back to default configuration) because : {}",
								e)
							)
						);
					}
				},
				Err(e) => {
					charlie_buffalo::push(
						&logger,
						vec![
							crate::LogLevel::INFO.into(),
							charlie_buffalo::Attr::new("component", "app").into(),
							charlie_buffalo::Attr::new("stage", "configuration").into(),
						],
						Some(
							&format!("error while reading configuration (this is not fatal, we falling back to default configuration) because : {}",
							e)
						)
					);
				}
			}
		} else {
			charlie_buffalo::push(
				&logger,
				vec![
					crate::LogLevel::DEBUG.into(),
					charlie_buffalo::Attr::new("component", "app").into(),
					charlie_buffalo::Attr::new("stage", "configuration").into(),
				],
				Some(&format!(
					"configuration file does not exists at {}, creating it with default value",
					&instructions.configuration_path
				)),
			);

			if let Some(folder) = &configuration_path.parent() {
				match std::fs::create_dir_all(folder) {
					Ok(_) => {
						if let Ok(bytes) = toml::to_vec(&result) {
							match std::fs::write(&instructions.configuration_path, bytes) {
								Ok(_) => {}
								Err(e) => {
									charlie_buffalo::push(
										&logger,
										vec![
											crate::LogLevel::INFO.into(),
											charlie_buffalo::Attr::new("component", "app").into(),
											charlie_buffalo::Attr::new("stage", "configuration")
												.into(),
										],
										Some(&format!(
											"can not create file {} (this is not fatal) because : {}",
											&instructions.configuration_path, e
										)),
									);
								}
							}
						}
					}
					Err(e) => {
						charlie_buffalo::push(
							&logger,
							vec![
								crate::LogLevel::INFO.into(),
								charlie_buffalo::Attr::new("component", "app").into(),
								charlie_buffalo::Attr::new("stage", "configuration").into(),
							],
							Some(&format!(
								"can not create folder {} (this is not fatal) because : {}",
								folder.display(),
								e
							)),
						);
					}
				}
			}
		};

		charlie_buffalo::push(
			&logger,
			vec![
				crate::LogLevel::DEBUG.into(),
				charlie_buffalo::Attr::new("component", "app").into(),
				charlie_buffalo::Attr::new("stage", "configuration").into(),
			],
			Some(&format!("config is : {:?}", &result)),
		);

		return result;
	}
}
