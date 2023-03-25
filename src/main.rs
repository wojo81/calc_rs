#![allow(non_camel_case_types)]

mod evaluating;
mod parsing;
mod scanning;

use evaluating::*;
use parsing::*;

fn main() {
    use std::io::Write;
    
    print!("> ");
    std::io::stdout().flush().unwrap();

    for line in std::io::stdin().lines() {
        let expression = parse(line.unwrap());

        println!("{}", evaluate(&expression));

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}