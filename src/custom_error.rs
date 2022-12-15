use std::{error, fmt};

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub struct Error {
    error: Option<Box<dyn error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub fn new_from_err<E: error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            error: Some(Box::new(e)),
        }
    }
}

impl<T: error::Error + Send + Sync + 'static> From<T> for Error {
    fn from(e: T) -> Self {
        Self::new_from_err(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bad error")
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(err) = self.error.as_ref() {
            writeln!(f, "{}", err)?;

            let mut current = err.source();

            while let Some(cause) = current {
                writeln!(f, "\nCaused by:\n\t{}", cause)?;

                current = cause.source();
            }
        }

        Ok(())
    }
}
