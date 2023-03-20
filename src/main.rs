#![allow(non_camel_case_types)]

#[derive(Clone)]
enum LexemeKind {
    identifier, number, operator, punctuation
}

#[derive(Clone)]
struct Lexeme {
    content: String,
    kind: LexemeKind,
}

impl Lexeme {
    fn new(content: String, kind: LexemeKind) -> Self {
        Self{content, kind}
    }
}

trait Source {
    fn get_current(&self) -> Lexeme;
    fn advance(&mut self);
    fn is_valid(&self) -> bool;
}

struct SimpleSource {
    lexemes: Vec<Lexeme>,
    index: usize,
}

impl SimpleSource {
    fn new(lexemes: Vec<Lexeme>) -> Self {
        Self{lexemes, index: 0}
    }
}

impl Source for SimpleSource {
    fn get_current(&self) -> Lexeme {
        self.lexemes[self.index].clone()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn is_valid(&self) -> bool {
        self.index != self.lexemes.len()
    }
}

struct StringSource {
    string: String,
    lexeme: Option<Lexeme>,
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
            lexeme: None,
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

    fn get_number(&self) -> Lexeme {
        let count = self.count(is_digit_or_dot);
        Lexeme::new(self.string[self.index..(self.index + count)].into(), LexemeKind::number)
    }

    fn get_single(&self, kind: LexemeKind) -> Lexeme {
        Lexeme::new(self.string[self.index..(self.index + 1)].into(), kind)
    }

    fn get_lexeme(&mut self) -> Option<Lexeme> {
        if self.view().is_empty() {
            None
        } else if self.view().starts_with(char::is_numeric) {
            Some(self.get_number())
        } else if self.view().starts_with(is_operator) {
            Some(self.get_single(LexemeKind::operator))
        } else if self.view().starts_with(is_punctuation) {
            Some(self.get_single(LexemeKind::punctuation))
        } else {
            panic!("unexpected character encountered")
        }
    }
}

impl Source for StringSource {
    fn get_current(&self) -> Lexeme {
        self.lexeme.clone().unwrap()
    }

    fn advance(&mut self) {
        self.skip_whitespace();
        let lexeme = self.get_lexeme();
        if let Some(lexeme) = &lexeme {
            self.index += lexeme.content.len();
        }
        self.lexeme = lexeme;
    }

    fn is_valid(&self) -> bool {
        self.lexeme.is_some()
    }
}

struct Yard {
    expression: Vec<Lexeme>,
    stack: Vec<Lexeme>,
}

impl Yard {
    fn new() -> Self {
        Self{expression: Vec::new(), stack: Vec::new()}
    }

    fn add_number(&mut self, number: Lexeme) {
        self.expression.push(number);
    }

    fn pop_higher_operator(&mut self, precedence: usize) -> Option<Lexeme> {
        if self.stack.last().filter(|stack_operator| precedence_of(&stack_operator.content) >= precedence).is_some() {
            self.stack.pop()
        } else {
            None
        }
    }

    fn pop_operator(&mut self) -> Option<Lexeme> {
        self.stack.pop()
    }

    fn add_operator(&mut self, operator: Lexeme) {
        let precedence = precedence_of(&operator.content);
        while let Some(stack_operator) = self.pop_higher_operator(precedence) {
            self.expression.push(stack_operator);
        }        
        self.stack.push(operator);
    }

    fn add_left_paren(&mut self, paren: Lexeme) {
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

fn handle_edge(yard: &mut Yard, lexeme: &Lexeme) -> bool {
    use LexemeKind::*;
    match lexeme.kind {
        number => {
            yard.add_number(lexeme.clone());
            true
        },
        punctuation => {
            match lexeme.content.as_str() {
                "(" => yard.add_left_paren(lexeme.clone()),
                ")" => panic!("did not expect ')'"),
                _ => panic!("unexpected token")
            }
            false
        },
        _ => panic!("wrong token")
    }
}

fn handle_middle(yard: &mut Yard, lexeme: &Lexeme) -> bool {
    use LexemeKind::*;
    match lexeme.kind {
        operator => {
            yard.add_operator(lexeme.clone());
            true
        },
        punctuation => {
            match lexeme.content.as_str() {
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

fn function_of(lexeme: &Lexeme) -> BinaryOperation {
    match lexeme.content.as_str() {
        "+" => |a, b| a + b,
        "-" => |a, b| a - b,
        "*" => |a, b| a * b,
        "/" => |a, b| a / b,
        _ => panic!("invalid operator")
    }    
}

fn evaluate(yard: &Yard) -> f32 {
    let mut slots = Vec::<f32>::new();
    for lexeme in &yard.expression {
        use LexemeKind::*;
        match lexeme.kind {
            number => slots.push(lexeme.content.parse().unwrap()),
            operator => {
                let right = slots.pop().unwrap();
                let left = *slots.last().unwrap();

                *slots.last_mut().unwrap() = function_of(lexeme)(left, right);
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
            let lexeme = source.get_current();
            if is_edge {
                if handle_edge(&mut yard, &lexeme) {
                    is_edge = false;
                }
            } else {
                if handle_middle(&mut yard, &lexeme) {
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