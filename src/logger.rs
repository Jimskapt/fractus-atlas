pub fn new(
	cli_instructions: &crate::cli_parsing::CliInstructions,
) -> charlie_buffalo::ConcurrentLogger {
	let mut logpath = std::path::PathBuf::from(&cli_instructions.working_folder);
	logpath.push("logs.msgpack");
	let logpath_for_buffer = logpath.clone();
	let logpath_for_drop = logpath;
	let show_debug = cli_instructions.debug;

	// This buffer is used to centralize receive of logs and write them asynchronously.
	let (buffer_tx, buffer_rx) = std::sync::mpsc::channel::<charlie_buffalo::Log>();
	std::thread::spawn(move || {
		let mut result: Vec<charlie_buffalo::Log> = rmp_serde::decode::from_slice(
			std::fs::read(&logpath_for_buffer)
				.unwrap_or_default()
				.as_slice(),
		)
		.unwrap_or_default();

		let mut do_write = false;

		loop {
			match buffer_rx.recv_timeout(std::time::Duration::from_millis(10)) {
				Ok(log) => {
					result.push(log);

					do_write = true;
				}
				Err(_) => {
					if do_write {
						std::fs::write(
							&logpath_for_buffer,
							rmp_serde::encode::to_vec(&result).unwrap(),
						)
						.unwrap();

						do_write = false;
					}
				}
			}
		}
	});

	let logger = charlie_buffalo::concurrent_logger_from(charlie_buffalo::Logger::new(
		charlie_buffalo::new_dispatcher(Box::from(move |log: charlie_buffalo::Log| {
			let mut new_log = log;

			if new_log.attributes.get("time").is_none() {
				new_log.attributes.insert(
					String::from("time"),
					format!("{}", chrono::offset::Local::now()),
				);
			}

			let log_int_value: usize = new_log
				.attributes
				.get("level")
				.unwrap_or(&charlie_buffalo::ValueAsString::as_string(
					&crate::LogLevel::INFO,
				))
				.parse()
				.unwrap_or_default();

			let debug_int_value: usize =
				charlie_buffalo::ValueAsString::as_string(&crate::LogLevel::DEBUG)
					.parse()
					.unwrap_or_default();

			if show_debug || log_int_value > debug_int_value {
				println!("{}", new_log);
			}

			buffer_tx.send(new_log).unwrap();
		})),
		charlie_buffalo::new_dropper(Box::from(move |_logger: &charlie_buffalo::Logger| {
			// The async buffer can't write this log before overall drop,
			// so we do this synchronously.
			let mut logs: Vec<charlie_buffalo::Log> = rmp_serde::decode::from_slice(
				std::fs::read(&logpath_for_drop)
					.unwrap_or_default()
					.as_slice(),
			)
			.unwrap_or_default();

			logs.push(charlie_buffalo::Log::from((
				vec![
					crate::LogLevel::DEBUG.into(),
					charlie_buffalo::Attr::new("stage", "stop").into(),
				],
				Some("stopping the app"),
			)));

			std::fs::write(&logpath_for_drop, rmp_serde::encode::to_vec(&logs).unwrap()).unwrap();

			println!(
				"\n(logs should be inside file {})\n",
				&logpath_for_drop.as_path().to_str().unwrap()
			);
		})),
	));

	return logger;
}

#[derive(serde::Serialize)]
pub enum LogLevel {
	DEBUG,
	INFO,
	WARN,
	ERROR,
	PANIC,
}

impl charlie_buffalo::ValueAsString for LogLevel {
	fn as_string(&self) -> String {
		format!(
			"{}",
			match self {
				LogLevel::DEBUG => 10,
				LogLevel::INFO => 20,
				LogLevel::WARN => 30,
				LogLevel::ERROR => 40,
				LogLevel::PANIC => 50,
			}
		)
	}
}

impl std::cmp::PartialEq<LogLevel> for &String {
	fn eq(&self, other: &LogLevel) -> bool {
		*self == &charlie_buffalo::ValueAsString::as_string(other)
	}
}

impl std::convert::Into<(String, String)> for LogLevel {
	fn into(self) -> (String, String) {
		return (
			String::from("level"),
			charlie_buffalo::ValueAsString::as_string(&self),
		);
	}
}
