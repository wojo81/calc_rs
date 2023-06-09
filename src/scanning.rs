use crate::error_handling::*;

#[derive(Clone, PartialEq, Eq)]
pub enum TokenKind {
    identifier, number, operator, punctuation
}

#[derive(Clone)]
pub struct Token {
    pub content: String,
    pub kind: TokenKind,
}

impl Token {
    fn new(content: String, kind: TokenKind) -> Self {
        Self{content, kind}
    }
}

pub struct StringScanner {
    string: String,
    index: usize,
}

fn is_operator(character: char) -> bool {
    match character {
        '+' | '-' | '*' | '/' | '^' | '=' => true,
        _ => false
    }
}

fn is_punctuation(character: char) -> bool {
    match character {
        '(' | ')' | ',' => true,
        _ => false
    }
}

fn is_digit_or_dot(character: char) -> bool {
    character.is_numeric() || character == '.'
}

impl StringScanner {
    pub fn new(string: String) -> Self {
        let mut scanner = Self {
            string,
            index: 0,
        };
        scanner.skip_whitespace();
        scanner
    }

    fn count_while<P: Fn(char) -> bool>(&self, predicate: P) -> usize {
        self.view().chars().take_while(|c| predicate(*c)).count()
    }

    fn view(&self) -> &str {
        &self.string[self.index..]
    }

    fn skip_whitespace(&mut self) {
        self.index += self.count_while(char::is_whitespace);
    }

    fn slice_while(&mut self, predicate: fn(char) -> bool) -> String {
        let count = self.count_while(predicate);
        let slice = self.view()[..count].to_string();
        self.index += count;
        slice
    }

    fn slice_many_as(&mut self, predicate: fn(char) -> bool, kind: TokenKind) -> Option<Token> {
        let slice = self.slice_while(predicate);
        if slice.is_empty() {
            None
        } else {
            Some(Token::new(slice, kind))
        }
    }

    fn slice_once_as(&mut self, predicate: fn(char) -> bool, kind: TokenKind) -> Option<Token> {
        if self.view().starts_with(predicate) {
            let slice = self.view()[..1].to_string();
            self.index += 1;
            Some(Token::new(slice, kind))
        } else {
            None
        }
    }

    fn peel_number(&mut self) -> Option<Token> {
        self.slice_many_as(is_digit_or_dot, TokenKind::number)
    }

    fn peel_operator(&mut self) -> Option<Token> {
        self.slice_once_as(is_operator, TokenKind::operator)
    }

    fn peel_punctuation(&mut self) -> Option<Token> {
        self.slice_once_as(is_punctuation, TokenKind::punctuation)
    }

    fn peel_identifier(&mut self) -> Option<Token> {
        self.slice_many_as(char::is_alphabetic, TokenKind::identifier)
    }

    pub fn is_empty(&self) -> bool {
        self.view().is_empty()
    }

    fn peel(&mut self) -> Option<Result<Token>> {
        if self.is_empty() {
            None
        } else if let Some(token) = self.peel_number() {
            Some(Ok(token))
        } else if let Some(token) = self.peel_operator() {
            Some(Ok(token))
        } else if let Some(token) = self.peel_punctuation() {
            Some(Ok(token))
        } else if let Some(token) = self.peel_identifier() {
            Some(Ok(token))
        } else {
            Some(Err(CalcError::invalid_character(self.view().chars().next().unwrap().into())))
        }
    }
}

impl Iterator for StringScanner {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let peeling = self.peel();
        self.skip_whitespace();
        peeling
    }
}