pub enum CalcError {
    character(InvalidCharacter),
    number(InvalidNumber),
    operator(InvalidOperator),
    did_not_expect(DidNotExpect),
    could_not_find(CouldNotFind),
}

impl ToString for CalcError {
    fn to_string(&self) -> String {
        use CalcError::*;
        match self {
            character(e) => e.to_string(),
            number(e) => e.to_string(),
            operator(e) => e.to_string(),
            did_not_expect(e) => e.to_string(),
            could_not_find(e) => e.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, CalcError>;

pub struct InvalidCharacter {
    content: String,
}

impl InvalidCharacter {
    pub fn new(content: String) -> Self {
        Self{content}
    }
}

impl ToString for InvalidCharacter {
    fn to_string(&self) -> String {
        format!("invalid character, '{}', encountered", self.content)
    }
}

pub struct InvalidNumber {
    content: String,
}

impl InvalidNumber {
    pub fn new(content: String) -> Self {
        Self{content}
    }
}

impl ToString for InvalidNumber {
    fn to_string(&self) -> String {
        format!("'{}' is not a valid number", self.content)
    }
}


pub struct InvalidOperator {
    operator: String,
}

impl InvalidOperator {
    pub fn new(operator: String) -> Self {
        Self{operator}
    }
}

impl ToString for InvalidOperator {
    fn to_string(&self) -> String {
        format!("the '{}' operator has been misplaced", self.operator)
    }
}


pub struct DidNotExpect {
    content: String,
}

impl DidNotExpect {
    pub fn new(content: String) -> Self {
        DidNotExpect{content}
    }
}

impl ToString for DidNotExpect {
    fn to_string(&self) -> String {
        format!("did not expect '{}'", self.content)
    }
}


pub struct CouldNotFind {
    content: String,
}

impl CouldNotFind {
    pub fn new(content: String) -> Self {
        Self{content}
    }
}

impl ToString for CouldNotFind {
    fn to_string(&self) -> String {
        format!("could not find '{}'", self.content)
    }
}


impl From<InvalidNumber> for CalcError {
    fn from(e: InvalidNumber) -> Self {
        CalcError::number(e)
    }
}

impl From<InvalidOperator> for CalcError {
    fn from(e: InvalidOperator) -> Self {
        CalcError::operator(e)
    }
}

impl From<InvalidCharacter> for CalcError {
    fn from(e: InvalidCharacter) -> Self {
        CalcError::character(e)
    }
}

impl From<DidNotExpect> for CalcError {
    fn from(e: DidNotExpect) -> Self {
        CalcError::did_not_expect(e)
    }
}

impl From<CouldNotFind> for CalcError {
    fn from(e: CouldNotFind) -> Self {
        CalcError::could_not_find(e)
    }
}