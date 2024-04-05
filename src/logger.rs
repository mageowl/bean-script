use std::collections::VecDeque;

pub enum Logger {
	Stdout,
	Backlog { backlog: VecDeque<String> },
	Callback { log: fn(String) },
}

impl Logger {
	pub fn log(&mut self, msg: String) {
		match self {
			Logger::Stdout => println!("{}", msg),
			Logger::Backlog { backlog } => backlog.push_back(msg),
			Logger::Callback { log } => log(msg),
		}
	}
}

impl Iterator for Logger {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Logger::Backlog { backlog } => backlog.pop_front(),
			_ => None,
		}
	}
}
