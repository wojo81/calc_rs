#![allow(non_camel_case_types)]

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
        let mut source = StringScanner::new(line.unwrap());

        let mut is_edge = true;
        let mut yard = Yard::new();

        while source.is_valid() {
            let token = source.get_current();
            if is_edge {
                if handle_edge(&mut yard, &token) {
                    is_edge = false;
                }
            } else {
                if handle_middle(&mut yard, &token) {
                    is_edge = true;
                }
            }
            source.advance();
        }
        yard.finish();

        println!("{}", evaluate(&yard));

        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}