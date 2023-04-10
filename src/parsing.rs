use crate::error_handling::*;
use crate::scanning::*;
use std::str::FromStr;

pub enum UnaryOperator {
    negative,
    positive,
}

impl UnaryOperator {
    pub fn call(&self, value: f32) -> f32 {
        use UnaryOperator::*;
        match self {
            positive => value,
            negative => -value,
        }
    }
}

impl FromStr for UnaryOperator {
    type Err = CalcError;

    fn from_str(s: &str) -> Result<Self> {
        use UnaryOperator::*;
        match s {
            "+" => Ok(positive),
            "-" => Ok(negative),
            _ => Err(InvalidOperator::new(s.into()).into()),
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
    type Err = CalcError;

    fn from_str(s: &str) -> Result<Self> {
        use BinaryOperator::*;
        match s {
            "+" => Ok(addition),
            "-" => Ok(subtraction),
            "*" => Ok(multiplication),
            "/" => Ok(division),
            "^" => Ok(exponentiation),
            _ => Err(InvalidOperator::new(s.into()).into()),
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
}

impl From<StackNode> for ExprNode {
    fn from(node: StackNode) -> Self {
        match node {
            StackNode::operator(operator) => ExprNode::operator(operator),
            _ => panic!("stack node could not be converted to an expression node"),
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

    fn add_number(&mut self, content: &str) -> Result<()> {
        self.expression.push(ExprNode::number(content.parse()
            .map_err(|_| CalcError::from(InvalidNumber::new(content.into())))? ));
        Ok(())
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

    fn add_operator(&mut self, content: &str, is_binary: bool) -> Result<()> {
        use Operator::*;
        use Punctuation::*;

        let operator;
        if is_binary {
            operator = StackNode::operator(binary(content.parse()?));
            let precedence = operator.precedence();
            while let Some(operator) = self.pop_higher_operator(precedence) {
                self.expression.push(ExprNode::operator(operator));
            }
        } else {
            match self.stack.last() {
                None | Some(StackNode::punctuation(paren)) | Some(StackNode::operator(binary(_))) => (),
                _ => return Err(InvalidOperator::new(content.into()).into()),
            }
            operator = StackNode::operator(unary(content.parse()?));
        }

        self.stack.push(operator);
        Ok(())
    }

    fn add_left_paren(&mut self) {
        self.stack.push(StackNode::punctuation(Punctuation::paren));
    }

    fn add_right_paren(&mut self) -> Result<()> {
        while let Some(stack_node) = self.stack.pop() {
            if let StackNode::punctuation(Punctuation::paren) = stack_node {
                return Ok(());
            }
            self.expression.push(stack_node.try_into().unwrap());
        }
        Err(CouldNotFind::new(")".into()).into())
    }

    pub fn finish(&mut self) -> Result<()> {
        while let Some(stack_node) = self.stack.pop() {
            if let StackNode::punctuation(Punctuation::paren) = stack_node {
                return Err(CouldNotFind::new("(".into()).into())
            }
            self.expression.push(stack_node.try_into().unwrap());
        }
        Ok(())
    }
}

struct ParsingStage {
    process: fn(&mut Yard, Token) -> Result<ParsingStage>,
}

const outer_stage: ParsingStage = ParsingStage {
    process: |yard, token| {
        use TokenKind::*;
        match token.kind {
            number => {
                yard.add_number(&token.content)?;
                Ok(inner_stage)
            },
            operator => {
                yard.add_operator(&token.content, false)?;
                Ok(outer_stage)
            },
            punctuation => {
                match token.content.as_str() {
                    "(" => yard.add_left_paren(),
                    _ => return Err(DidNotExpect::new(token.content).into()),
                }
                Ok(outer_stage)
            },
            _ => Err(DidNotExpect::new(token.content).into())
        }
    }
};

const inner_stage: ParsingStage = ParsingStage {
    process: |yard, token| {
        use TokenKind::*;
        match token.kind {
            operator => {
                yard.add_operator(&token.content, true)?;
                Ok(outer_stage)
            },
            punctuation => {
                match token.content.as_str() {
                    ")" => yard.add_right_paren()?,
                    _ => return Err(DidNotExpect::new(token.content).into()),
                }
                Ok(inner_stage)
            },
            _ => Err(DidNotExpect::new(token.content).into())
        }
    }
};

pub fn parse<T: Iterator<Item = Result<Token>>>(scanner: T) -> Result<Vec<ExprNode>> {
    let mut yard = Yard::new();
    let mut stage = outer_stage;

    for token in scanner {
        stage = (stage.process)(&mut yard, token?)?;
    }
    yard.finish()?;
    Ok(yard.expression)
}