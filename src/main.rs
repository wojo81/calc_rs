#![allow(nonstandard_style)]

mod error_handling;
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
        match parse(StringScanner::new(line.unwrap())) {
            Ok(expression) => println!("{}", evaluate(&expression)),
            Err(e) => println!("Error, {}", e.to_string()),
        }

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}