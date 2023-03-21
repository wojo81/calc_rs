#![allow(non_camel_case_types)]

#[derive(Clone)]
enum TokenKind {
    identifier, number, operator, punctuation
}

#[derive(Clone)]
struct Token {
    content: String,
    kind: TokenKind,
}

impl Token {
    fn new(content: String, kind: TokenKind) -> Self {
        Self{content, kind}
    }
}

trait Source {
    fn get_current(&self) -> Token;
    fn advance(&mut self);
    fn is_valid(&self) -> bool;
}

struct SimpleSource {
    tokens: Vec<Token>,
    index: usize,
}

impl SimpleSource {
    fn new(tokens: Vec<Token>) -> Self {
        Self{tokens, index: 0}
    }
}

impl Source for SimpleSource {
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

struct StringSource {
    string: String,
    token: Option<Token>,
    index: usize,
}

fn is_operator(character: char) -> bool {
    match character {
        '+' | '-' | '*' | '/' => true,
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

impl StringSource {
    fn new(string: String) -> Self {
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

impl Source for StringSource {
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

struct Yard {
    expression: Vec<Token>,
    stack: Vec<Token>,
}

impl Yard {
    fn new() -> Self {
        Self{expression: Vec::new(), stack: Vec::new()}
    }

    fn add_number(&mut self, number: Token) {
        self.expression.push(number);
    }

    fn pop_higher_operator(&mut self, precedence: usize) -> Option<Token> {
        if self.stack.last().filter(|stack_operator| precedence_of(&stack_operator.content) >= precedence).is_some() {
            self.stack.pop()
        } else {
            None
        }
    }

    fn pop_operator(&mut self) -> Option<Token> {
        self.stack.pop()
    }

    fn add_operator(&mut self, operator: Token) {
        let precedence = precedence_of(&operator.content);
        while let Some(stack_operator) = self.pop_higher_operator(precedence) {
            self.expression.push(stack_operator);
        }        
        self.stack.push(operator);
    }

    fn add_left_paren(&mut self, paren: Token) {
        self.stack.push(paren);
    }

    fn add_right_paren(&mut self) {
        while let Some(stack_operator) = self.pop_operator() {
            if stack_operator.content == "(" {
                return;
            }
            self.expression.push(stack_operator);
        }
        panic!("Could not find matching '('");
    }

    fn finish(&mut self) {
        while let Some(operator) = self.stack.pop() {
            if operator.content == "(" {
                panic!("Could not find matching ')'");
            }
            self.expression.push(operator);
        }
    }
}

fn precedence_of(operator: &str) -> usize {
    match operator {
        "+" | "-" => 1,
        "*" | "/" => 2,
        "(" => 0,
        _ => panic!("invalid operator")
    }
}

fn handle_edge(yard: &mut Yard, token: &Token) -> bool {
    use TokenKind::*;
    match token.kind {
        number => {
            yard.add_number(token.clone());
            true
        },
        punctuation => {
            match token.content.as_str() {
                "(" => yard.add_left_paren(token.clone()),
                ")" => panic!("did not expect ')'"),
                _ => panic!("unexpected token")
            }
            false
        },
        _ => panic!("wrong token")
    }
}

fn handle_middle(yard: &mut Yard, token: &Token) -> bool {
    use TokenKind::*;
    match token.kind {
        operator => {
            yard.add_operator(token.clone());
            true
        },
        punctuation => {
            match token.content.as_str() {
                ")" => yard.add_right_paren(),
                "(" => panic!("did not expect '('"),
                _ => panic!("unexpected token")
            }
            false
        },
        _ => panic!("wrong token")
    }
}

type BinaryOperation = fn(f32, f32) -> f32;

fn function_of(token: &Token) -> BinaryOperation {
    match token.content.as_str() {
        "+" => |a, b| a + b,
        "-" => |a, b| a - b,
        "*" => |a, b| a * b,
        "/" => |a, b| a / b,
        _ => panic!("invalid operator")
    }    
}

fn evaluate(yard: &Yard) -> f32 {
    let mut slots = Vec::<f32>::new();
    for token in &yard.expression {
        use TokenKind::*;
        match token.kind {
            number => slots.push(token.content.parse().unwrap()),
            operator => {
                let right = slots.pop().unwrap();
                let left = *slots.last().unwrap();

                *slots.last_mut().unwrap() = function_of(token)(left, right);
            },
            _ => panic!("wrong token"),
        }
    }
    *slots.first().unwrap()
}

fn main() {
    use std::io::Write;
    
    print!("> ");
    std::io::stdout().flush().unwrap();

    for line in std::io::stdin().lines() {
        let mut source = StringSource::new(line.unwrap());

        let mut is_edge = true;
        let mut yard = Yard::new();

        while source.is_valid() {
            let token = source.get_current();
            if is_edge {
                if handle_edge(&mut yard, &token) {
                    is_edge = false;
                }
            } else {
                if handle_middle(&mut yard, &token) {
                    is_edge = true;
                }
            }
            source.advance();
        }
        yard.finish();

        println!("{}", evaluate(&yard));

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}