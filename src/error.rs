use std::{ error::Error, fmt::Display };

#[derive(Debug)]
pub struct BeanError {
    source: Option<(usize, usize)>,
    message: String,
}

impl Display for BeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message);
        if let Some((line, col)) = self.source {
            write!(f, "\n\t(line {}, col {})", line, col);
        }
        Ok(())
    }
}

impl Error for BeanError {}
