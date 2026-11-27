use std::ops::Range;

use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Range<usize>,
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("unexpected number: {0:?}")]
    NumError(Range<usize>),

    #[error("not ended string: {0:?}")]
    StrError(Range<usize>),
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("not closed: {0:?}")]
    NotClosedError(Range<usize>),
    #[error("extra close: {0:?}")]
    ExtraCloseError(Range<usize>),
    #[error("nothing to separate: {0:?}")]
    NothingToSeparate(Range<usize>),
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("not closed: {0:?}")]
    NotClosedError(Range<usize>),
}
