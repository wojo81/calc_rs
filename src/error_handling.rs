use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalcError {
    #[error("invalid character, '{0}', enountered")]
    invalid_character(String),

    #[error("'{0}' is not a valid number")]
    invalid_number(String),

    #[error("the '{0}' operator has been misplaced")]
    invalid_operator(String),

    #[error("did not expect '{0}'")]
    did_not_expect(String),

    #[error("could not find '{0}'")]
    could_not_find(String),

    #[error("identifier, '{0}', is not defined")]
    undefined(String),

    #[error("expression ended abruptly")]
    abrupt_end,
}

pub type Result<T> = std::result::Result<T, CalcError>;