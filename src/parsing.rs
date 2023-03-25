use crate::scanning::*;

pub struct Yard {
    pub expression: Vec<Token>,
    stack: Vec<Token>,
}

impl Yard {
    pub fn new() -> Self {
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

    pub fn finish(&mut self) {
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

pub fn handle_edge(yard: &mut Yard, token: &Token) -> bool {
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

pub fn handle_middle(yard: &mut Yard, token: &Token) -> bool {
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

pub fn function_of(token: &Token) -> BinaryOperation {
    match token.content.as_str() {
        "+" => |a, b| a + b,
        "-" => |a, b| a - b,
        "*" => |a, b| a * b,
        "/" => |a, b| a / b,
        _ => panic!("invalid operator")
    }    
}