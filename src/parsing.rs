use crate::scanning::*;
use std::str::FromStr;

pub enum UnaryOperator {
    negative,
    positive,
}

impl UnaryOperator {
    fn call(&self, value: f32) -> f32 {
        use UnaryOperator::*;
        match self {
            positive => value,
            negative => -value,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidOperator;

impl FromStr for UnaryOperator {
    type Err = InvalidOperator;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UnaryOperator::*;
        match s {
            "+" => Ok(positive),
            "-" => Ok(negative),
            _ => Err(InvalidOperator),
        }
    }
}

pub enum BinaryOperator {
    addition,
    subtraction,
    multiplication,
    division,
    exponentiation,
}

impl BinaryOperator {
    pub fn call(&self, left: f32, right: f32) -> f32 {
        use BinaryOperator::*;

        match self {
            addition => left + right,
            subtraction => left - right,
            multiplication => left * right,
            division => left / right,
            exponentiation => left.powf(right),
        }
    }
}

impl FromStr for BinaryOperator {
    type Err = InvalidOperator;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BinaryOperator::*;
        match s {
            "+" => Ok(addition),
            "-" => Ok(subtraction),
            "*" => Ok(multiplication),
            "/" => Ok(division),
            "^" => Ok(exponentiation),
            _ => Err(InvalidOperator),
        }
    }
}

pub enum Operator {
    unary(UnaryOperator),
    binary(BinaryOperator),
}

pub enum Punctuation {
    paren,
}

pub enum StackNode {
    operator(Operator),
    punctuation(Punctuation),
}

impl StackNode {
    fn precedence(&self) -> i32 {
        use Operator::*;
        use BinaryOperator::*;
        match self {
            Self::punctuation(_) => 0,
            Self::operator(binary(operator)) =>
                match operator {
                    addition | subtraction => 1,
                    multiplication | division => 2,
                    exponentiation => 3,
                },
            Self::operator(unary(_)) => 1,
        }
    }
}

pub enum ExprNode {
    number(f32),
    operator(Operator),
    punctuation(Punctuation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidNode;

impl TryFrom<StackNode> for ExprNode {
    type Error = InvalidNode;

    fn try_from(node: StackNode) -> Result<Self, Self::Error> {
        match node {
            StackNode::operator(operator) => Ok(ExprNode::operator(operator)),
            _ => Err(InvalidNode),
        }
    }
}

pub struct Yard {
    expression: Vec<ExprNode>,
    stack: Vec<StackNode>,
}

impl Yard {
    pub fn new() -> Self {
        Self{expression: Vec::new(), stack: Vec::new()}
    }

    fn add_number(&mut self, content: &str) {
        self.expression.push(ExprNode::number(content.parse().unwrap()));
    }

    fn pop_higher_operator(&mut self, precedence: i32) -> Option<Operator> {
        if self.stack.last().filter(|operator| operator.precedence() >= precedence && precedence != 3).is_some() {
            if let Some(StackNode::operator(operator)) = self.stack.pop() {
                Some(operator)
            } else {
                panic!("popped node not an operator!");
            }
        } else {
            None
        }
    }

    fn add_operator(&mut self, content: &str, is_binary: bool) {
        use Operator::*;

        let operator;
        if is_binary {
            operator = StackNode::operator(binary(content.parse().unwrap()));
        } else {
            operator = StackNode::operator(unary(content.parse().unwrap()));
        }

        let precedence = operator.precedence();
        while let Some(operator) = self.pop_higher_operator(precedence) {
            self.expression.push(ExprNode::operator(operator));
        }
        self.stack.push(operator);
    }

    fn add_left_paren(&mut self) {
        self.stack.push(StackNode::punctuation(Punctuation::paren));
    }

    fn add_right_paren(&mut self) {
        while let Some(stack_node) = self.stack.pop() {
            if let StackNode::punctuation(Punctuation::paren) = stack_node {
                return;
            }
            self.expression.push(stack_node.try_into().unwrap());
        }
        panic!("Could not find matching '('");
    }

    pub fn finish(&mut self) {
        while let Some(stack_node) = self.stack.pop() {
            if let StackNode::punctuation(Punctuation::paren) = stack_node {
                panic!("Could not find matching ')'");
            }
            self.expression.push(stack_node.try_into().unwrap());
        }
    }
}

struct ParsingStage {
    process: fn(&mut Yard, Token) -> ParsingStage,
}

const outer_stage: ParsingStage = ParsingStage {
    process: |yard, token| {
        use TokenKind::*;
        match token.kind {
            number => {
                yard.add_number(&token.content);
                inner_stage
            }
            punctuation => {
                match token.content.as_str() {
                    "(" => yard.add_left_paren(),
                    ")" => panic!("did not expect ')'"),
                    _ => panic!("unexpected token")
                }
                outer_stage
            },
            _ => panic!("wrong token")
        }
    }
};

const inner_stage: ParsingStage = ParsingStage {
    process: |yard, token| {
        use TokenKind::*;
        match token.kind {
            operator => {
                yard.add_operator(&token.content, true);
                outer_stage
            },
            punctuation => {
                match token.content.as_str() {
                    ")" => yard.add_right_paren(),
                    "(" => panic!("did not expect '('"),
                    _ => panic!("unexpected token")
                }
                inner_stage
            },
            _ => panic!("wrong token")
        }
    }
};

pub fn parse<T: Iterator<Item = Token>>(scanner: T) -> Vec<ExprNode> {
    let mut yard = Yard::new();
    let mut stage = outer_stage;

    for token in scanner {
        stage = (stage.process)(&mut yard, token);
    }
    yard.finish();
    yard.expression
}