use crate::parsing::*;

pub fn evaluate(expression: &Vec<ExprNode>) -> f32 {
    let mut slots = Vec::<f32>::new();
    for node in expression {
        match node {
            ExprNode::value(value) => slots.push(*value),

            ExprNode::cast(cast) => {
                let value = slots.pop().unwrap();
                slots.push((cast.action)(value));
            },

            ExprNode::tie(tie) => {
                let right = slots.pop().unwrap();
                let left = slots.pop().unwrap();
                slots.push((tie.action)(left, right));
            },

            ExprNode::knot(knot) => {
                let mut arguments = Vec::with_capacity(knot.count as usize);
                for _ in 0..knot.count {
                    arguments.push(slots.pop().unwrap());
                }
                slots.push((knot.action)(arguments));
            },
        }
    }
    *slots.first().unwrap()
}