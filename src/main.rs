use std::io::{self, BufRead, Write};

use arithmetic_parser::evaluate;

fn run(input: &str) {
    match evaluate(input) {
        Ok(value) => println!("{value}"),
        Err(e) => eprintln!("{e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // `arithmetic-parser "2 + 3 * 4"` evaluates once; with no arguments, start a REPL.
    if !args.is_empty() {
        run(&args.join(" "));
        return;
    }

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) if line.trim().is_empty() => continue,
            Ok(_) => run(&line),
        }
    }
}
