use std::env;

use std::io::{Write, stdout};
use std::sync::atomic::{AtomicBool, Ordering};

mod execution;
mod native_functions;
mod parsing;
mod resolver;
mod scanning;

use execution::interpret;
use parsing::{Parser, ParserOutput};
use scanning::*;

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(script_file: &str) {
    let script = std::fs::read_to_string(script_file).unwrap();
    run(&script);
    if HAD_ERROR.load(Ordering::Relaxed) {
        std::process::exit(65);
    }
}

fn run_prompt() {
    let stdin = std::io::stdin();
    let mut buf = String::new();
    let mut stdout = stdout().lock();

    loop {
        buf.clear();
        print!("> ");
        stdout.flush().unwrap();

        let bytes_read = stdin.read_line(&mut buf).unwrap();
        if bytes_read == 0 {
            break;
        }
        run(&buf);
    }
}

fn run(script: &str) {
    let mut scanner = Scanner::new(script);

    let tokens: Vec<Token> = scanner.scan_tokens();

    let parser = Parser::new(tokens);

    let parser_output = parser.parse();

    match parser_output {
        ParserOutput::Good(program) => interpret(program),
        ParserOutput::Bad(errors) => {
            for e in errors {
                println!("{}", e);
            }
        }
    }
}

fn report(line: usize, error_where: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, error_where, message);
    HAD_ERROR.store(true, Ordering::Relaxed);
}

fn report_raw(message: &str) {
    println!("{}", message);
    HAD_ERROR.store(true, Ordering::Relaxed);
}
