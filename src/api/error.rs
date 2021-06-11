use actix_web::{http::StatusCode, HttpResponse, ResponseError};

use crate::api::response::SimpleResponse;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Error(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Database(msg) => write!(f, "DatabaseError: {}", msg),
            ErrorKind::Error(e) => e.fmt(f),
            ErrorKind::Simple(s) => write!(f, "SimpleError: {} ({})", s.code, s.message),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error {
            kind: ErrorKind::Error(error),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self.kind {
            ErrorKind::Simple(s) => StatusCode::from_u16(s.code).unwrap(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        let mut t = SimpleResponse {
            code: 500,
            message: String::new(),
        };
        let res = match &self.kind {
            ErrorKind::Database(m) => {
                t.message = m.to_string();
                &t
            }
            ErrorKind::Error(e) => {
                t.message = e.to_string();
                &t
            }
            ErrorKind::Simple(s) => s,
        };
        HttpResponse::build(self.status_code()).json(res)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Error(Box<dyn std::error::Error>),
    Database(String),
    Simple(SimpleResponse),
}

impl Error {
    pub fn map_from(error: Box<dyn std::error::Error>) -> Self {
        Error {
            kind: ErrorKind::Error(error),
        }
    }
    pub fn from_simple(error: SimpleResponse) -> Self {
        Error {
            kind: ErrorKind::Simple(error),
        }
    }
}
