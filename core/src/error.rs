use jsona::Error as JsonaError;
use jsona::Position;
use serde::{Deserialize, Serialize};
use thiserror::Error as TError;

#[derive(TError, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Error {
    #[error("{0}")]
    Jsona(JsonaError),
    #[error("{} at line {} col {}", .info, .position.line, .position.col)]
    InvalidAnnotation {
        info: String,
        scope: Vec<String>,
        position: Position,
    },
    #[error("{} at line {} col {}", .info, .position.line, .position.col)]
    InvalidAst {
        info: String,
        scope: Vec<String>,
        position: Position,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn invalid_annotation<T: ToString>(
        info: T,
        name: &str,
        scope: &[&str],
        position: Position,
    ) -> Self {
        let mut scope: Vec<String> = scope.iter().map(|v| v.to_string()).collect();
        let mut scope_path = scope.join(".");
        scope.push(name.to_owned());
        scope_path.push_str("@");
        scope_path.push_str(name);
        let info = format!("annotaion({}) {}", scope_path, info.to_string());
        Error::InvalidAnnotation {
            info,
            scope,
            position,
        }
    }
    pub fn invalid_ast<T: ToString>(info: T, scope: &[&str], position: Position) -> Self {
        let scope: Vec<String> = scope.iter().map(|v| v.to_string()).collect();
        let scope_path = scope.join(".");
        let info = format!("value({}) {}", scope_path, info.to_string());
        Error::InvalidAst {
            info,
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
