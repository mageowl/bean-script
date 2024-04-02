use std::{
	fmt::{self, Display, Formatter},
	path::PathBuf,
};

#[derive(Debug)]
pub enum ErrorSource {
	Unknown,
	Character(usize, usize),
	Full {
		line: usize,
		col: usize,
		path: PathBuf,
	},
}

#[derive(Debug)]
pub struct Error {
	source: ErrorSource,
	msg: String,
}

impl Error {
	pub fn new(msg: &str, source: ErrorSource) -> Self {
		Self {
			msg: String::from(msg),
			source,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(&self.msg)?;
		f.write_str("\n\t")?;
		match &self.source {
			ErrorSource::Unknown => f.write_str("(unknown)")?,
			ErrorSource::Character(l, c) => f.write_fmt(format_args!("({}:{})", l, c))?,
			ErrorSource::Full { line, col, path } => f.write_fmt(format_args!(
				"({}:{}:{})",
				path.to_str().unwrap(),
				line,
				col
			))?,
		}
		Ok(())
	}
}
