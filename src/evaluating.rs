use crate::parsing::*;

pub fn evaluate(expression: &Vec<ExprNode>) -> f32 {
    let mut slots = Vec::<f32>::new();
    for node in expression {
        match node {
            ExprNode::number(value) => slots.push(*value),
            ExprNode::operator(Operator::binary(operator)) => {
                let right = slots.pop().unwrap();
                let left = *slots.last().unwrap();

                *slots.last_mut().unwrap() = operator.call(left, right);
            },
            ExprNode::operator(Operator::unary(operator)) => {
                let value = slots.last().unwrap();
                *slots.last_mut().unwrap() = operator.call(*value);
            },
        }
    }
    *slots.first().unwrap()
}