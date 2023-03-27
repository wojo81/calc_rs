#![allow(nonstandard_style)]

mod evaluating;
mod parsing;
mod scanning;

use evaluating::*;
use parsing::*;
use scanning::*;

fn main() {
    use std::io::Write;

    print!("> ");
    std::io::stdout().flush().unwrap();

    for line in std::io::stdin().lines() {
        let expression = parse(StringScanner::new(line.unwrap()));

        println!("{}", evaluate(&expression));

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}