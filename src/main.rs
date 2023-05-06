#![allow(nonstandard_style)]

mod error_handling;
mod evaluating;
mod parsing;
mod scanning;

use evaluating::*;
use parsing::*;
use scanning::*;

use std::collections::HashMap;

fn main() {
    use std::io::Write;

    print!("> ");
    std::io::stdout().flush().unwrap();

    let mut variables = HashMap::<String, f32>::new();

    for line in std::io::stdin().lines() {
        let scanner = StringScanner::new(line.unwrap());

        if scanner.is_empty() {
            break;
        }

        match parse(scanner, &mut variables) {
            Ok(expression) => println!("{}", evaluate(&expression, &mut variables)),
            Err(e) => println!("Error, {}", e.to_string()),
        }

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}