mod builtin_words;

use std::collections::HashMap;
use console;
use std::io::{self, Write};
// use std::str::Chars;
use crate::builtin_words::{FINAL, ACCEPTABLE};

/// The main function for the Wordle game, implement your own logic here
pub const ALPHABET: &[char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
    'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);
    if is_tty {
        println!(
            "I am in a tty. Please print {}!",
            console::style("colorful characters").bold().blink().blue()
        );
    } else {
        // println!("I am not in a tty. Please print according to test requirements!");
    }
    //
    // if is_tty {
    //     print!("{}", console::style("Your name: ").bold().red());
    //     io::stdout().flush().unwrap();
    // }
    // let mut line = String::new();
    // io::stdin().read_line(&mut line)?;
    // println!("Welcome to wordle, {}!", line.trim());
    // // example: print arguments
    // print!("Command line arguments: ");
    // for arg in std::env::args() {
    //     print!("{} ", arg);
    // }
    //
    // println!("");
    // TODO: parse the arguments in `args`
    let mut alphabet_color: Vec<Color> = vec![];
    for _i in 0..26 {
        let temp = Color::X;
        alphabet_color.push(temp);
    }
    let mut word_to_guess = String::new();
    match std::env::args().nth(1){
        Some(arg)=>{
            match &arg[..]{
                "-w"|"--word"=>{
                    word_to_guess=std::env::args().nth(2).expect("did not input");
                }
                _=>unimplemented!()
            }
        }
        None=>{
            io::stdin().read_line(&mut word_to_guess)?;
            word_to_guess.pop();
        }
    }
    let mut is_success: bool = false;
    let mut i = 0;
    while i < 6 {
        //Guess 6 times
        match guess_1(&word_to_guess, &mut alphabet_color) {
            Err(error) => {
                match error {
                    Error::InvalidWord => { println!("{}", error.to_string()) }
                    Error::AlreadyCorrect => {
                        i+=1;
                        is_success = true;
                        print!(" ");
                        for i in &alphabet_color {
                            print!("{}", i.to_string());
                        }
                        println!();
                        println!("{} {}", error.to_string(), i);
                        break;
                    }
                }
            }
            Ok(_ok) => {
                i+=1;
                print!(" ");
                for i in &alphabet_color {
                    print!("{}", i.to_string());
                }
                println!();
            }
        }
    }
    if !is_success {
        println!("FAILED {}", word_to_guess.to_ascii_uppercase());
    }
    Ok(())
}


fn match_result(guess_word: &String, word_to_guess: &String, alphabet: &mut Vec<Color>) -> Result<(), Error> {
    // Calculate the color, print a string of 5 letters, and return updated alphabet_color
    // First find G, ignore them, then match last letters one by one (first 5 letters)
    // For alphabet, use a vec of 5 to record the condition of 5 letters

    let mut word_result: Vec<Color> = vec![];
    let mut char_to_ignore: Vec<i32> = vec![];
    for i in 0..5 {
        if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(i as usize) {
            char_to_ignore.push(i);
        }
    }
    // println!("{:?}", char_to_ignore);
    let mut char_to_ignore_to_guess = char_to_ignore.clone();
    for i in 0..5 {
        if char_to_ignore.contains(&i) {
            word_result.push(Color::G);
            continue;
        }
        let mut is_in: bool = false;
        for j in 0..5 {
            if char_to_ignore_to_guess.contains(&j) {
                continue;
            }
            if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(j as usize) {
                word_result.push(Color::Y);
                char_to_ignore_to_guess.push(j);
                is_in = true;
                break;
            }
        }
        if !is_in {
            word_result.push(Color::R);
        }
    }
    for i in &word_result {
        print!("{}", i.to_string());
    }
    let color_grade = HashMap::from([
        ("G".to_string(), 4),
        ("Y".to_string(), 3),
        ("R".to_string(), 2),
        ("X".to_string(), 1)
    ]);
    for i in 0..26 {
        for j in 0..5 {
            if ALPHABET[i] == guess_word.chars().nth(j).unwrap() {
                if color_grade.get(
                    &alphabet[i].to_string()) < color_grade.get(&word_result[j].to_string()
                ) {
                    alphabet[i] = word_result[j].clone();
                }
            }
        }
    }
    if guess_word == word_to_guess { return Err(Error::AlreadyCorrect); }
    return Ok(());
}

fn guess_1(word_to_guess: &String, alphabet: &mut Vec<Color>) -> Result<(), Error> {
    //do guess operation once, and return updated alphabet_color
    let mut word = String::new();
    io::stdin().read_line(&mut word).expect("cannot read");
    word.pop();
    for i in ACCEPTABLE {
        if word == i.to_string() {
            // println!("valid!");
            // is_acceptable = true;
            return match_result(&word, word_to_guess, alphabet);
        }
    }
    return Err(Error::InvalidWord);
}


enum Error {
    InvalidWord,
    AlreadyCorrect,
}

#[derive(Debug)]
#[derive(Clone)]
enum Color {
    R,
    Y,
    G,
    X,
}

impl Color {
    fn to_string(&self) -> String {
        return match &self {
            Color::R => { "R".to_string() }
            Color::Y => { "Y".to_string() }
            Color::G => { "G".to_string() }
            Color::X => { "X".to_string() }
        };
    }
    fn clone(&self) -> Color {
        match self {
            Color::R => { Color::R }
            Color::Y => { Color::Y }
            Color::G => { Color::G }
            Color::X => { Color::X }
        }
    }
}

impl Error {
    fn to_string(&self) -> String {
        return match &self {
            Error::InvalidWord => { "INVALID".to_string() }
            Error::AlreadyCorrect => { "CORRECT".to_string() }
        };
    }
}