use crate::parsing::*;
use crate::scanning::*;

pub fn evaluate(yard: &Yard) -> f32 {
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