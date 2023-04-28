use crate::scanning::*;
use crate::error_handling::*;
use std::collections::HashMap;

pub struct Cast {
    pub action: fn(f32) -> f32,
}

pub struct Tie {
    pub action: fn(f32, f32) -> f32,
}

pub struct Knot {
    pub count: u32,
    pub action: fn(Vec<f32>) -> f32,
}

enum Precedence {
    low, medium, high,
}

impl Precedence {
    fn precedes(&self, other: &Self) -> bool {
        use Precedence::*;
        match (self, other) {
            (_, high) => false,
            (high, _) => true,
            (medium, _) => true,
            (_, medium) => false,
            _ => true,
        }
    }
}

#[derive(Clone)]
enum Function {
    positive, negative,
    floor, ceil, round,
    sin, cos, tan,
    asin, acos, atan,
    todeg, torad,
    log, ln,
    sqrt, cbrt,
    abs,
}

impl Into<ExprNode> for Function {
    fn into(self) -> ExprNode {
        ExprNode::cast(Cast {action: self.call()})
    }
}

impl Function {
    fn from_operator(content: &str) -> Result<Self> {
        use Function::*;
        match content {
            "+" => Ok(positive),
            "-" => Ok(negative),
            _ => Err(InvalidOperator::new(content.into()).into())
        }
    }

    fn from_identifier(content: &str) -> Option<Self> {
        use Function::*;
        match content {
            "floor" => Some(floor),
            "ceil" => Some(ceil),
            "round" => Some(round),
            "sin" => Some(sin),
            "cos" => Some(cos),
            "tan" => Some(tan),
            "asin" => Some(asin),
            "acos" => Some(acos),
            "atan" => Some(atan),
            "todeg" => Some(todeg),
            "torad" => Some(torad),
            "log" => Some(log),
            "ln" => Some(ln),
            "sqrt" => Some(sqrt),
            "cbrt" => Some(cbrt),
            "abs" => Some(abs),
            _ => None
        }
    }

    fn call(self) -> fn(f32) -> f32 {
        use Function::*;
        match self {
            positive => |n| n,
            negative => |n| -n,
            floor => f32::floor,
            ceil => f32::ceil,
            round => f32::round,
            sin => f32::sin,
            cos => f32::cos,
            tan => f32::tan,
            asin => f32::asin,
            acos => f32::acos,
            atan => f32::atan,
            todeg => f32::to_degrees,
            torad => f32::to_radians,
            log => f32::log10,
            ln => f32::ln,
            sqrt => f32::sqrt,
            cbrt => f32::cbrt,
            abs => f32::abs,
        }
    }

    fn precedence(&self) -> Precedence {
        match self {
            Self::positive | Self::negative => Precedence::low,
            _ => Precedence::high,
        }
    }

    fn preceding(&self, precedence: &Precedence) -> Option<ExprNode> {
        if self.precedence().precedes(precedence) {
            Some(self.clone().into())
        } else {
            None
        }
    }
}

#[derive(Clone)]
enum BinaryFunction {
    addition, subtraction,
    multiplication, division,
    exponentiation,
}

impl Into<ExprNode> for BinaryFunction {
    fn into(self) -> ExprNode {
        ExprNode::tie(Tie {action: self.call()})
    }
}

impl BinaryFunction {
    fn from_operator(content: &str) -> Result<BinaryFunction> {
        use BinaryFunction::*;

        match content {
            "+" => Ok(addition),
            "-" => Ok(subtraction),
            "*" => Ok(multiplication),
            "/" => Ok(division),
            "^" => Ok(exponentiation),
            _ => Err(InvalidOperator::new(content.into()).into())
        }
    }

    fn call(self) -> fn(f32, f32) -> f32 {
        use BinaryFunction::*;
        match self {
            addition => |a, b| a + b,
            subtraction => |a, b| a - b,
            multiplication => |a, b| a * b,
            division => |a, b| a / b,
            exponentiation => |a, b| a.powf(b),
        }
    }

    fn precedence(&self) -> Precedence {
        use BinaryFunction::*;
        match self {
            addition | subtraction => Precedence::low,
            multiplication | division => Precedence::medium,
            exponentiation => Precedence::high,
        }
    }

    fn preceding(&self, precedence: &Precedence) -> Option<ExprNode> {
        if self.precedence().precedes(precedence) {
            Some(self.clone().into())
        } else {
            None
        }
    }
}

enum VariedFunction {
    min, max, avg,
}

pub enum ExprNode {
    value(f32),
    cast(Cast),
    tie(Tie),
    knot(Knot),
}

enum StackNode {
    function(Function),
    binary_function(BinaryFunction),
    varied_function(VariedFunction),
    section { is_nested: bool, is_argument: bool },
}

type Cause = fn(&Token) -> bool;
type Effect = fn(&mut Context, &mut Yard, &Token) -> Result<()>;

struct Rule {
    cause: Cause,
    effect: Effect,
}

impl Rule {
    fn applies(&self, token: &Token) -> Option<Effect> {
        if (self.cause)(token) {
            Some(self.effect)
        } else {
            None
        }
    }
}

struct Ruleset {
    rules: Vec<Vec<Rule>>,
}

const value_placing: Rule = Rule {
    cause: |token| {
        if let TokenKind::number = token.kind {
            true
        } else {
            false
        }
    },
    effect: |context, feeder, token| {
        context.active_ruleset = ActiveRuleset::binding;
        feeder.expression.push(ExprNode::value(token.content.parse()
            .map_err(|_| CalcError::from(InvalidNumber::new(token.content.clone())))? ));
        Ok(())
    }
};

const operator_placing: Rule = Rule {
    cause: |token| {
        if let TokenKind::operator = token.kind {
            true
        } else {
            false
        }
    },
    effect: |_context, feeder, token| {
        let operator = Function::from_operator(&token.content)?;
        Ok(feeder.stack.push(StackNode::function(operator)))
    }
};

const paren_placing: Rule = Rule {
    cause: |token| {
        token.content == "("
    },
    effect: |context, feeder, _token| {
        context.binding.push(vec![paren_binding]);
        feeder.stack.push(StackNode::section {is_nested: context.is_nested, is_argument: context.is_list});
        context.is_nested = true;
        context.is_list = false;
        Ok(())
    }
};

const paren_binding: Rule = Rule {
    cause: |token| {
        token.content == ")"
    },
    effect: |context, feeder, _token| {
        while let Some(node) = feeder.stack.pop() {
            match node {
                StackNode::section { is_nested, is_argument: is_list } => {
                    if !is_nested {
                        context.is_nested = false;
                        context.binding.reset();
                    }
                    break;
                },
                StackNode::function(node)  => feeder.expression.push(node.into()),
                StackNode::binary_function(node) => feeder.expression.push(node.into()),
                _ => (),
            }
        }
        Ok(())
    }
};

const operator_binding: Rule = Rule {
    cause: |token| {
        if let TokenKind::operator = token.kind {
            true
        } else {
            false
        }
    },
    effect: |context, feeder, token| {
        context.active_ruleset = ActiveRuleset::placing;
        let operator = BinaryFunction::from_operator(&token.content)?;
        let precedence = operator.precedence();
        while let Some(node) = feeder.pop_preceding(&precedence) {
            feeder.expression.push(node)
        }
        Ok(feeder.stack.push(StackNode::binary_function(operator)))
    }
};

const identifier_placing: Rule = Rule {
    cause: |token| {
        if let TokenKind::identifier = token.kind {
            true
        } else {
            false
        }
    },
    effect: |context, feeder, token| {
        if let Some(constant) = context.constants.get(&token.content) {
            context.active_ruleset = ActiveRuleset::binding;
            Ok(feeder.expression.push(ExprNode::value(*constant)))
        } else if let Some(function) = Function::from_identifier(&token.content) {
            Ok(feeder.stack.push(StackNode::function(function)))
        } else {
            Err(Undefined::new(token.content.clone()).into())
        }
    }
};

impl Ruleset {
    fn placing() -> Self {
        Self {
            rules: vec![
                vec![
                    value_placing,
                    operator_placing,
                    paren_placing,
                    identifier_placing,
                ]
            ]
        }
    }

    fn binding() -> Self {
        Self {
            rules: vec![
                vec![
                    operator_binding,
                ]
            ]
        }
    }

    fn applies(&self, token: &Token) -> Result<Effect> {
        for rule in self.rules.iter().rev().flatten() {
            if let Some(effect) = rule.applies(token) {
                return Ok(effect);
            }
        }
        Err(DidNotExpect::new(token.content.clone().into()).into())
    }

    fn reset(&mut self) {
        self.rules.truncate(1);
    }

    fn push(&mut self, rules: Vec<Rule>) {
        self.rules.push(rules);
    }
}

#[derive(Clone)]
enum ActiveRuleset {
    placing, binding,
}

struct Context {
    placing: Ruleset,
    binding: Ruleset,
    active_ruleset: ActiveRuleset,
    constants: HashMap<String, f32>,
    is_nested: bool,
    is_list: bool,
}

fn create_constants() -> HashMap<String, f32> {
    HashMap::from([
        ("pi".into(), std::f32::consts::PI),
        ("e".into(), std::f32::consts::E)
    ])
}

impl Context {
    fn new() -> Self {
        Self {
            placing: Ruleset::placing(),
            binding: Ruleset::binding(),
            active_ruleset: ActiveRuleset::placing,
            constants: create_constants(),
            is_nested: false,
            is_list: false,
        }
    }

    fn apply(&mut self, yard: &mut Yard, token: Token) -> Result<()> {
        let effect = match self.active_ruleset.clone() {
            ActiveRuleset::placing => self.placing.applies(&token),
            ActiveRuleset::binding => self.binding.applies(&token),
        }?;

        effect(self, yard, &token)
    }
}

struct Yard {
    expression: Vec<ExprNode>,
    stack: Vec<StackNode>,
}

impl Yard {
    fn new() -> Self {
        Self {
            expression: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn get_preceding(&mut self, precedence: &Precedence) -> Option<ExprNode> {
        if let Some(node) = self.stack.last() {
            match node {
                StackNode::function(function) => function.preceding(precedence),
                StackNode::binary_function(function) => function.preceding(precedence),
                _ => None
            }
        } else {
            None
        }
    }

    fn pop_preceding(&mut self, precedence: &Precedence) -> Option<ExprNode> {
        if let Some(node) = self.get_preceding(precedence) {
            self.stack.pop();
            Some(node)
        } else {
            None
        }
    }

    pub fn finalize(&mut self) -> Result<()> {
        while let Some(node) = self.stack.pop() {
            match node {
                StackNode::section{..} => return Err(CouldNotFind::new(")".into()).into()),
                StackNode::function(function) => self.expression.push(function.into()),
                StackNode::binary_function(function) => self.expression.push(function.into()),
                _ => panic!("temporary")
            }
        }
        Ok(())
    }
}

pub fn parse<T: Iterator<Item = Result<Token>>>(scanner: T) -> Result<Vec<ExprNode>> {
    let mut yard = Yard::new();
    let mut context = Context::new();

    for token in scanner {
        context.apply(&mut yard, token?)?;
    }
    yard.finalize()?;

    Ok(yard.expression)
}