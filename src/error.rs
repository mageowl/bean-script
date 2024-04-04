use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ErrorSource {
	Internal,
	Builtin(String),
	Line(usize),
	File(String),
}

#[derive(Debug)]
pub struct Error {
	trace: Vec<ErrorSource>,
	msg: String,
}

impl Error {
	pub fn new(msg: &str, source: ErrorSource) -> Self {
		Self {
			msg: String::from(msg),
			trace: vec![source],
		}
	}

	pub fn get_source(&self) -> String {
		let mut ln = None;
		let mut file = None;
		for source in &self.trace {
			match source {
				ErrorSource::Line(line) => {
					if ln.is_none() {
						ln = Some(line)
					}
				}
				ErrorSource::File(path) => {
					if file.is_none() {
						file = Some(path)
					}
				}
				_ => (),
			}
			if ln.is_some() && file.is_some() {
				break;
			}
		}
		format!("{}:{}", file.unwrap_or(&String::from("input")), ln.unwrap())
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(&self.msg)?;
		f.write_fmt(format_args!(
			"\n\x1b[36m->\x1b[0m {}\x1b[3m",
			self.get_source()
		))?;
		for source in &self.trace {
			match source {
				ErrorSource::Internal => (),
				ErrorSource::Builtin(name) => {
					f.write_str("\n\t")?;
					f.write_fmt(format_args!("(builtin {})", name))?
				}
				ErrorSource::Line(ln) => {
					f.write_str("\n\t")?;
					f.write_fmt(format_args!("(line {})", ln))?;
				}
				ErrorSource::File(path) => {
					f.write_str("\n\t")?;
					f.write_fmt(format_args!("(file {})", path))?
				}
			}
		}
		f.write_str("\x1b[0m")?;
		Ok(())
	}
}

pub trait BeanResult {
	fn trace(self, source: ErrorSource) -> Self;
}

impl<T> BeanResult for Result<T, Error> {
	fn trace(mut self, source: ErrorSource) -> Self {
		if let Err(error) = &mut self {
			error.trace.push(source)
		}
		self
	}
}

impl BeanResult for Error {
	fn trace(mut self, source: ErrorSource) -> Self {
		self.trace.push(source);
		self
	}
}
