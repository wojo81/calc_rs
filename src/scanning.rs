
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

pub trait Scanner {
    fn get_current(&self) -> Token;
    fn advance(&mut self);
    fn is_valid(&self) -> bool;
}

pub struct SimpleScanner {
    tokens: Vec<Token>,
    index: usize,
}

impl SimpleScanner {
    fn new(tokens: Vec<Token>) -> Self {
        Self{tokens, index: 0}
    }
}

impl Scanner for SimpleScanner {
    fn get_current(&self) -> Token {
        self.tokens[self.index].clone()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn is_valid(&self) -> bool {
        self.index != self.tokens.len()
    }
}

pub struct StringScanner {
    string: String,
    token: Option<Token>,
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
        let mut source = Self {
            string,
            token: None,
            index: 0,
        };
        source.advance();
        source
    }

    fn count<P: Fn(char) -> bool>(&self, predicate: P) -> usize {
        let mut chars = self.string.chars().skip(self.index);
        let mut counter = 0;
        while let Some(c) = chars.next() {
            if !predicate(c) {
                break;
            }
            counter += 1;
        }
        counter
    }

    fn view(&self) -> &str {
        &self.string[self.index..]
    }

    fn skip_whitespace(&mut self) {
        let count = self.count(char::is_whitespace);
        self.index += count;
    }

    fn get_number(&self) -> Token {
        let count = self.count(is_digit_or_dot);
        Token::new(self.string[self.index..(self.index + count)].into(), TokenKind::number)
    }

    fn get_single(&self, kind: TokenKind) -> Token {
        Token::new(self.string[self.index..(self.index + 1)].into(), kind)
    }

    fn get_token(&mut self) -> Option<Token> {
        if self.view().is_empty() {
            None
        } else if self.view().starts_with(char::is_numeric) {
            Some(self.get_number())
        } else if self.view().starts_with(is_operator) {
            Some(self.get_single(TokenKind::operator))
        } else if self.view().starts_with(is_punctuation) {
            Some(self.get_single(TokenKind::punctuation))
        } else {
            panic!("unexpected character encountered")
        }
    }
}

impl Scanner for StringScanner {
    fn get_current(&self) -> Token {
        self.token.clone().unwrap()
    }

    fn advance(&mut self) {
        self.skip_whitespace();
        let token = self.get_token();
        if let Some(token) = &token {
            self.index += token.content.len();
        }
        self.token = token;
    }

    fn is_valid(&self) -> bool {
        self.token.is_some()
    }
}