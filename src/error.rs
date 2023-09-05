use actix_web::{http::StatusCode, ResponseError};

/// A possible error value occurred when loading i18n resources.
#[derive(Debug, thiserror::Error)]
pub enum I18NError {
    /// Fluent error.
    #[error("fluent: {}", .0[0])]
    Fluent(Vec<fluent::FluentError>),

    /// Fluent FTL parser error.
    #[error("fluent parser: {}", .0[0])]
    FluentParser(Vec<fluent_syntax::parser::ParserError>),

    /// There is no value in the message.
    #[error("no value")]
    FluentNoValue,

    /// Message id was not found.
    #[error("msg not found: `{id}`")]
    FluentMessageNotFound {
        /// Message id
        id: String,
    },

    /// Invalid language id.
    #[error("invalid language id: {0}")]
    LanguageIdentifier(#[from] unic_langid::LanguageIdentifierError),

    /// Io error
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl ResponseError for I18NError {
    #[inline]
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
