mod builtin_words;

use console;
use std::io::{self, Write};
use std::io::ErrorKind::TimedOut;
use crate::builtin_words::{FINAL, ACCEPTABLE};

/// The main function for the Wordle game, implement your own logic here

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);

    if is_tty {
        println!(
            "I am in a tty. Please print {}!",
            console::style("colorful characters").bold().blink().blue()
        );
    } else {
        println!("I am not in a tty. Please print according to test requirements!");
    }

    if is_tty {
        print!("{}", console::style("Your name: ").bold().red());
        io::stdout().flush().unwrap();
    }
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    println!("Welcome to wordle, {}!", line.trim());

    // example: print arguments
    print!("Command line arguments: ");
    for arg in std::env::args() {
        print!("{} ", arg);
    }
    println!("");
    // TODO: parse the arguments in `args`

    Ok(())
}

fn match_result(word:String)->String{
    // Calculate the color, and return a string of 5 letters
    unimplemented!();
}

fn guess_1() -> Result<i32, Error> {
    let mut n = 0;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let mut is_acceptable: bool = false;
    for i in ACCEPTABLE {
        if line == i.to_string() {
            is_acceptable = true;
            print!("{}",match_result(line));
            break;
        }
    }
    return if is_acceptable {
        Ok(n)
    } else {
        Err(Error::InvalidWord)
    }
}

enum Error {
    InvalidWord
}
enum Color{
    R,
    Y,
    G,
    X
}

impl Color {
    fn to_string(&self)->String{
        return match self {
            Color::R => {"R".to_string()}
            Color::Y => {"Y".to_string()}
            Color::G => {"G".to_string()}
            Color::X => {"X".to_string()}
        }
    }
}