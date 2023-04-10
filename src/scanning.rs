use crate::error_handling::*;

#[derive(Clone)]
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
        '+' | '-' | '*' | '/' | '^' => true,
        _ => false
    }
}

fn is_punctuation(character: char) -> bool {
    match character {
        '(' | ')' => true,
        _ => false
    }
}

fn is_digit_or_dot(character: char) -> bool {
    character.is_numeric() || character == '.'
}

impl StringScanner {
    pub fn new(string: String) -> Self {
        Self {
            string,
            index: 0,
        }
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

    fn get_number(&self) -> Token {
        let count = self.count_while(is_digit_or_dot);
        Token::new(self.string[self.index..(self.index + count)].into(), TokenKind::number)
    }

    fn get_single(&self, kind: TokenKind) -> Token {
        Token::new(self.string[self.index..(self.index + 1)].into(), kind)
    }

    fn get_token(&mut self) -> Option<Result<Token>> {
        if self.view().is_empty() {
            None
        } else if self.view().starts_with(char::is_numeric) {
            Some(Ok(self.get_number()))
        } else if self.view().starts_with(is_operator) {
            Some(Ok(self.get_single(TokenKind::operator)))
        } else if self.view().starts_with(is_punctuation) {
            Some(Ok(self.get_single(TokenKind::punctuation)))
        } else {
            Some(Err(InvalidCharacter::new(self.string.chars().next().unwrap().into()).into()))
        }
    }
}

impl Iterator for StringScanner {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let token = self.get_token();
        match &token {
            Some(Ok(token)) => self.index += token.content.len(),
            _ => ()
        }
        token
    }
}