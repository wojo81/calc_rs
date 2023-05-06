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
    pub action: fn(Vec<f32>) -> f32,
    pub count: u32,
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
            _ => Err(CalcError::invalid_operator(content.into()))
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
    fn from_operator(content: &str) -> Result<Self> {
        use BinaryFunction::*;

        match content {
            "+" => Ok(addition),
            "-" => Ok(subtraction),
            "*" => Ok(multiplication),
            "/" => Ok(division),
            "^" => Ok(exponentiation),
            _ => Err(CalcError::invalid_operator(content.into()))
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

impl VariedFunction {
    fn from_identifier(content: &str) -> Option<Self> {
        use VariedFunction::*;
        match content {
            "min" => Some(min),
            "max" => Some(max),
            "avg" => Some(avg),
            _ => None
        }
    }

    fn call(self) -> fn(Vec<f32>) -> f32 {
        use VariedFunction::*;
        match self {
            min => |values| values.iter().fold(f32::MAX, |a, b| a.min(*b)),
            max => |values| values.iter().fold(f32::MIN, |a, b| a.max(*b)),
            avg => |values| values.iter().sum::<f32>() / values.len() as f32,
        }
    }
}

pub enum ExprNode {
    value(f32),
    cast(Cast),
    tie(Tie),
    knot(Knot),
    assign(String),
}

impl ExprNode {
    fn varied(function: VariedFunction, count: u32) -> Self {
        Self::knot(Knot {action: function.call(), count})
    }
}

enum StackNode {
    function(Function),
    binary_function(BinaryFunction),
    varied_function(VariedFunction, u32),
    section(Enclosure),
    variable(String),
    assign(String),
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

const value_placing: Rule = Rule {
    cause: |token| {
        if let TokenKind::number = token.kind {
            true
        } else {
            false
        }
    },
    effect: |context, yard, token| {
        context.active_ruleset = ActiveRuleset::binding;
        yard.expression.push(ExprNode::value(token.content.parse()
            .map_err(|_| CalcError::invalid_number(token.content.clone()))? ));
        Ok(())
    }
};

const operator_placing: Rule = Rule {
    cause: |token| {
        token.kind == TokenKind::operator
    },
    effect: |_context, yard, token| {
        let operator = Function::from_operator(&token.content)?;
        Ok(yard.stack.push(StackNode::function(operator)))
    }
};

const paren_placing: Rule = Rule {
    cause: |token| {
        token.content == "("
    },
    effect: |context, yard, _token| {
        yard.stack.push(StackNode::section(context.enclosure.clone()));
        context.enclose(Enclosure::nested);
        Ok(())
    }
};

const paren_binding: Rule = Rule {
    cause: |token| {
        token.content == ")"
    },
    effect: |context, yard, _token| {
        while let Some(node) = yard.stack.pop() {
            match node {
                StackNode::section(enclosure) => {
                    context.enclose(enclosure);
                    break;
                },
                StackNode::function(node)  => yard.expression.push(node.into()),
                StackNode::binary_function(node) => yard.expression.push(node.into()),
                _ => (),
            }
        }
        Ok(())
    }
};

const operator_binding: Rule = Rule {
    cause: |token| {
        token.kind == TokenKind::operator
    },
    effect: |context, yard, token| {
        context.active_ruleset = ActiveRuleset::placing;
        let operator = BinaryFunction::from_operator(&token.content)?;
        let precedence = operator.precedence();
        while let Some(node) = yard.pop_preceding(&precedence) {
            yard.expression.push(node)
        }
        Ok(yard.stack.push(StackNode::binary_function(operator)))
    }
};

const identifier_placing: Rule = Rule {
    cause: |token| {
        token.kind == TokenKind::identifier
    },
    effect: |context, yard, token| {
        if let Some(constant) = context.constants.get(&token.content) {
            context.active_ruleset = ActiveRuleset::binding;
            Ok(yard.expression.push(ExprNode::value(*constant)))
        } else if let Some(variable) = context.variables.get(&token.content) {
            context.active_ruleset = ActiveRuleset::binding;
            Ok(yard.expression.push(ExprNode::value(*variable)))
        } else if let Some(function) = Function::from_identifier(&token.content) {
            Ok(yard.stack.push(StackNode::function(function)))
        } else if let Some(function) = VariedFunction::from_identifier(&token.content) {
            context.placing.push(vec![list_placing]);
            Ok(yard.stack.push(StackNode::varied_function(function, 0)))
        } else {
            Err(CalcError::undefined(token.content.clone()))
        }
    }
};

const list_placing: Rule = Rule {
    cause: |_token| {
        true
    },
    effect: |context, yard, token| {
        if token.content != "(" {
            Err(CalcError::did_not_expect(token.content.clone()))
        } else {
            context.placing.reset();
            context.enclose(Enclosure::listed);
            Ok(yard.stack.push(StackNode::section(context.enclosure.clone())))
        }
    }
};

const arg_binding: Rule = Rule {
    cause: |token| {
        token.content == ","
    },
    effect: |context, yard, _token| {
        context.active_ruleset = ActiveRuleset::placing;
        while let Some(node) = yard.stack.pop() {
            match node {
                StackNode::section(enclosure) => {
                    if let Some(StackNode::varied_function(function, count)) = yard.stack.pop() {
                        yard.stack.push(StackNode::varied_function(function, count + 1));
                        yard.stack.push(StackNode::section(enclosure));
                    }
                    break;
                },
                StackNode::function(node)  => yard.expression.push(node.into()),
                StackNode::binary_function(node) => yard.expression.push(node.into()),
                _ => (),
            }
        }
        Ok(())
    }
};

const list_binding: Rule = Rule {
    cause: |token| {
        token.content == ")"
    },
    effect: |_context, yard, _token| {
        while let Some(node) = yard.stack.pop() {
            match node {
                StackNode::section(_) => {
                    if let Some(StackNode::varied_function(function, count)) = yard.stack.pop() {
                        yard.expression.push(ExprNode::varied(function, count + 1));
                    }
                    break;
                },
                StackNode::function(node)  => yard.expression.push(node.into()),
                StackNode::binary_function(node) => yard.expression.push(node.into()),
                _ => (),
            }
        }
        Ok(())
    }
};

const assign_placing: Rule = Rule {
    cause: |token| {
        token.kind == TokenKind::identifier
    },
    effect: |context, yard, token| {
        if let Some(constant) = context.constants.get(&token.content) {
            context.active_ruleset = ActiveRuleset::binding;
            Ok(yard.expression.push(ExprNode::value(*constant)))
        } else if let Some(function) = Function::from_identifier(&token.content) {
            Ok(yard.stack.push(StackNode::function(function)))
        } else if let Some(function) = VariedFunction::from_identifier(&token.content) {
            context.placing.push(vec![list_placing]);
            Ok(yard.stack.push(StackNode::varied_function(function, 0)))
        } else {
            context.active_ruleset = ActiveRuleset::binding;
            context.binding.push(vec![assign_binding]);
            Ok(yard.stack.push(StackNode::variable(token.content.clone())))
        }
    }
};

const assign_binding: Rule = Rule {
    cause: |token| {
        token.kind == TokenKind::operator
    },
    effect: |context, yard, token| {
        if let Some(StackNode::variable(identifier)) = yard.stack.pop() {
            if token.content == "=" {
                context.active_ruleset = ActiveRuleset::placing;
                yard.stack.push(StackNode::assign(identifier));
                Ok(context.binding.reset())
            } else if let Some(value) = context.variables.get(&identifier) {
                yard.expression.push(ExprNode::value(*value));
                (operator_binding.effect)(context, yard, token)
            } else {
                Err(CalcError::undefined(identifier))
            }
        } else {
            panic!("Expected variable at top of stack");
        }
    }
};

struct Ruleset {
    rules: Vec<Vec<Rule>>,
}

impl Ruleset {
    fn placing() -> Self {
        Self {
            rules: vec![
                vec![
                    value_placing,
                    operator_placing,
                    paren_placing,
                    identifier_placing,
                ],
                vec![assign_placing],
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
        Err(CalcError::did_not_expect(token.content.clone().into()))
    }

    fn reset(&mut self) {
        self.rules.truncate(1);
    }

    fn push(&mut self, rules: Vec<Rule>) {
        self.rules.push(rules);
    }
}

#[derive(Clone, PartialEq, Eq)]
enum ActiveRuleset {
    placing, binding,
}

#[derive(Clone, PartialEq, Eq)]
enum Enclosure {
    open, nested, listed
}

struct Context<'a> {
    placing: Ruleset,
    binding: Ruleset,
    active_ruleset: ActiveRuleset,
    constants: HashMap<String, f32>,
    variables: &'a mut HashMap<String, f32>,
    enclosure: Enclosure,
}

fn create_constants() -> HashMap<String, f32> {
    HashMap::from([
        ("pi".into(), std::f32::consts::PI),
        ("e".into(), std::f32::consts::E)
    ])
}

impl<'a> Context<'a> {
    fn new(variables: &'a mut HashMap<String, f32>) -> Self {
        Self {
            placing: Ruleset::placing(),
            binding: Ruleset::binding(),
            active_ruleset: ActiveRuleset::placing,
            constants: create_constants(),
            variables: variables,
            enclosure: Enclosure::open,
        }
    }

    fn apply(&mut self, yard: &mut Yard, token: Token) -> Result<()> {
        let effect = match self.active_ruleset.clone() {
            ActiveRuleset::placing => self.placing.applies(&token),
            ActiveRuleset::binding => self.binding.applies(&token),
        }?;

        effect(self, yard, &token)
    }

    fn enclose(&mut self, enclosure: Enclosure) {
        if self.enclosure != enclosure {
            self.placing.reset();
            self.binding.reset();
            if enclosure == Enclosure::nested {
                self.binding.push(vec![paren_binding])
            } else if enclosure == Enclosure::listed {
                self.binding.push(vec![arg_binding, list_binding])
            }
            self.enclosure = enclosure;
        }
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

    pub fn finalize(&mut self, context: &Context) -> Result<()> {
        if context.active_ruleset == ActiveRuleset::placing {
            return Err(CalcError::abrupt_end);
        }
        while let Some(node) = self.stack.pop() {
            match node {
                StackNode::section{..} => return Err(CalcError::could_not_find(")".into())),
                StackNode::function(function) => self.expression.push(function.into()),
                StackNode::binary_function(function) => self.expression.push(function.into()),
                StackNode::varied_function(..) => panic!("did not expect varied function"),
                StackNode::variable(identifier) =>
                    self.expression.push(
                        ExprNode::value(*context.variables.get(&identifier)
                            .ok_or_else(|| CalcError::undefined(identifier.clone()))?)),
                StackNode::assign(identifier) => self.expression.push(ExprNode::assign(identifier)),
            }
        }
        Ok(())
    }
}

pub fn parse<T: Iterator<Item = Result<Token>>>(scanner: T, variables: &mut HashMap<String, f32>) -> Result<Vec<ExprNode>> {
    let mut yard = Yard::new();
    let mut context = Context::new(variables);

    let mut is_first_token = true;
    for token in scanner {
        context.apply(&mut yard, token?)?;
        if is_first_token {
            context.placing.reset();
            is_first_token = false;
        }
    }
    yard.finalize(&context)?;

    Ok(yard.expression)
}