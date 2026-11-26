use std::ops::Range;

use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct PositionedString(pub String, pub Range<usize>);

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("unexpected number: {0:?}")]
    NumError(PositionedString),

    #[error("not ended string: {0:?}")]
    StrError(PositionedString),
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("not closed")]
    NotClosedError,
    #[error("extra token after closed: {0:?}")]
    ExtraTokenAfterClosed(PositionedString),
}
