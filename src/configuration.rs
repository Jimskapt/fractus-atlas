#[derive(Debug, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Configuration {
	pub background: String,
}
impl Default for Configuration {
	fn default() -> Configuration {
		Configuration {
			background: String::from("#FFFFFF"),
		}
	}
}
impl std::convert::From<&crate::cli_parsing::CliInstructions> for Configuration {
	fn from(instructions: &crate::cli_parsing::CliInstructions) -> Self {
		let mut result = Configuration::default();

		let configuration_path = std::path::PathBuf::from(&instructions.configuration_path);

		if configuration_path.exists() {
			let configuration = std::fs::read_to_string(&instructions.configuration_path);

			match configuration {
				Ok(configuration) => match toml::from_str(&configuration) {
					Ok(configuration) => {
						result = configuration;
					}
					Err(e) => {
						println!(
                                "INFO: error while parsing configuration, falling back to default configuration (this is not fatal) : {}",
                                e
                            );
					}
				},
				Err(e) => {
					println!(
                        "INFO: error while reading configuration, falling back to default configuration (this is not fatal) : {}",
                        e
                    );
				}
			}
		} else {
			if instructions.debug {
				println!(
                    "DEBUG: configuration file does not exists at {}, creating it with default value",
                    &instructions.configuration_path
                );
			}

			if let Some(folder) = &configuration_path.parent() {
				match std::fs::create_dir_all(folder) {
					Ok(_) => match std::fs::write(
						&instructions.configuration_path,
						toml::to_vec(&result).unwrap(),
					) {
						Ok(_) => {}
						Err(e) => {
							println!(
								"INFO: can not create file {:?} (this is not fatal) : {}",
								&instructions.configuration_path, e
							);
						}
					},
					Err(e) => {
						println!(
							"INFO: can not create folder {:?} (this is not fatal) : {}",
							&folder, e
						);
					}
				}
			}
		};

		if instructions.debug {
			println!("DEBUG: {:?}", &result);
		}

		return result;
	}
}
