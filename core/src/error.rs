use jsona::Error as JsonaError;
use jsona::Position;
use thiserror::Error as TError;

/// errors that openapi functions may return
#[derive(TError, Debug)]
pub enum Error {
    #[error("{0}")]
    Jsona(JsonaError),
    #[error("annotation in scope {} at line {} col {} {}", .scope, .position.line, .position.col, .message)]
    InvalidAnnotation {
        message: String,
        scope: String,
        position: Position,
    },
    #[error("value {} in scope at line {} col {} {}", .scope, .position.line, .position.col, .message)]
    InvalidAst {
        message: String,
        scope: String,
        position: Position,
    },
}

impl Error {
    pub fn invalid_annotation<T: ToString>(
        message: T,
        name: &str,
        scope: &[&str],
        position: Position,
    ) -> Self {
        let scope = if scope.len() > 0 {
            format!("{}@{}", scope.join("."), name)
        } else {
            format!("@{}", name)
        };
        Error::InvalidAnnotation {
            message: message.to_string(),
            scope,
            position,
        }
    }
    pub fn invalid_ast<T: ToString>(message: T, scope: &[&str], position: Position) -> Self {
        let scope = scope.join(".");
        Error::InvalidAst {
            message: message.to_string(),
            scope,
            position,
        }
    }
}

impl From<JsonaError> for Error {
    fn from(e: JsonaError) -> Self {
        Error::Jsona(e)
    }
}
