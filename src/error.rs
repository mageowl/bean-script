use std::fmt::Display;

#[derive(Debug)]
pub struct BeanError {
    source: Option<(usize, usize)>,
    msg: String,
}

impl BeanError {
    pub fn new(msg: &str, source: Option<(usize, usize)>) -> Self {
        Self { msg: String::from(msg), source }
    }
}

impl Display for BeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)?;
        if let Some((line, col)) = self.source {
            write!(f, "\n\t(line {}, col {})", line, col)?;
        }
        Ok(())
    }
}
