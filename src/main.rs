mod builtin_words;

use std::collections::HashMap;
use console;
use std::io::{self, Write};
// use std::str::Chars;
use crate::builtin_words::{FINAL, ACCEPTABLE};
use rand::seq::{IteratorRandom};

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
    }
    // TODO: parse the arguments in `args`
    //initialize alphabet
    let mut alphabet_color: Vec<Color> = vec![];
    for _i in 0..26 {
        let temp = Color::X;
        alphabet_color.push(temp);
    }
    let mut word_to_guess = String::new();
    let mut is_difficult: bool = false;
    // println!("{:?}",std::env::args());
    mode_analyze(&mut word_to_guess, &mut is_difficult);
    if word_to_guess.is_empty() {
        io::stdin().read_line(&mut word_to_guess)?;
        word_to_guess.pop();
    }
    word_to_guess = word_to_guess.to_ascii_lowercase();
    let mut is_success: bool = false;
    let mut i = 0;
    let mut already_guessed: Vec<(i32, char)> = vec![];
    // HashMap
    while i < 6 {
        //Guess 6 times
        match guess_1(&word_to_guess, &mut alphabet_color, is_difficult, &mut already_guessed) {
            Err(error) => {
                match error {
                    Error::InvalidWord => { println!("{}", error.to_string()) }
                    Error::AlreadyCorrect => {
                        i += 1;
                        is_success = true;
                        print_alphabet(is_tty, &mut alphabet_color);
                        println!("{} {}", error.to_string(), i);
                        break;
                    }
                }
            }
            Ok(_ok) => {
                i += 1;

                print_alphabet(is_tty, &mut alphabet_color);
            }
        }
    }
    if !is_success {
        if is_tty {
            println!(
                "{} {}",
                console::style("FAILED").red(),
                console::style(word_to_guess.to_ascii_uppercase()).green().italic()
            );
        } else {
            println!("FAILED {}", word_to_guess.to_ascii_uppercase());
        }
    }
    Ok(())
}

fn mode_analyze(word_to_guess: &mut String, is_difficult: &mut bool) {
    let mut num_args = 0;
    loop {
        //loop to analyze args
        match std::env::args().nth(num_args) {
            None => break,
            Some(arg) => {
                // println!("{arg}");
                match &arg[..] {
                    "-w" | "--word" => {
                        *word_to_guess = std::env::args().nth(num_args + 1).expect("did not input word");
                    }
                    "-r" | "--random" => {
                        *word_to_guess = FINAL.iter().choose(&mut rand::thread_rng()).unwrap().to_string();
                    }
                    "-D" | "--difficult" => {
                        *is_difficult = true
                    }
                    _ => {}
                }
            }
        }
        num_args += 1;
    }
}

fn print_alphabet(is_tty: bool, alphabet_color: &Vec<Color>) {
    if is_tty {
        for i in 0..26 {
            print!("{}",
                   match alphabet_color[i] {
                       Color::Y => console::style(ALPHABET[i].to_ascii_uppercase()).yellow(),
                       Color::R => console::style(ALPHABET[i].to_ascii_uppercase()).white(),
                       Color::G => console::style(ALPHABET[i].to_ascii_uppercase()).green(),
                       Color::X => console::style(ALPHABET[i].to_ascii_uppercase()).black().bright()
                   }
            )
        }
    } else {
        for i in alphabet_color {
            print!("{}", i.to_string());
        }
    }
    println!();
}


fn match_result(guess_word: &String,
                word_to_guess: &String,
                alphabet: &mut Vec<Color>,
                is_difficult: bool,
                already_guessed_position: &mut Vec<(i32, char)>, ) -> Result<(), Error> {
    // Calculate the color, print a string of 5 letters, and return updated alphabet_color
    // First find G, ignore them, then match last letters one by one (first 5 letters)
    // For alphabet, use a vec of 5 to record the condition of 5 letters
    if is_difficult {
        for temp in already_guessed_position.iter() {
            if guess_word.chars().nth(temp.0 as usize).unwrap() != temp.1 {
                return Err(Error::InvalidWord);
            }
        }//letters already correct cannot change
        for i in 0..26 {
            if let  Color::Y=alphabet[i]{
                if !guess_word.contains(ALPHABET[i]){
                    return Err(Error::InvalidWord);
                }
            }
        }//letters in wrong position must be used
    }
    let mut word_result: Vec<Color> = vec![];
    let mut char_to_ignore: Vec<i32> = vec![];
    for i in 0..5 {
        if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(i as usize) {
            char_to_ignore.push(i);
            already_guessed_position.push((i, guess_word.chars().nth(i as usize).unwrap()))
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
    for i in 0..5 {
        if atty::is(atty::Stream::Stdout) {
            print!("{}",
                   match word_result[i] {
                       Color::Y => console::style(guess_word.chars().nth(i).unwrap()).yellow(),
                       Color::R => console::style(guess_word.chars().nth(i).unwrap()).black().bright(),
                       Color::G => console::style(guess_word.chars().nth(i).unwrap()).green(),
                       Color::X => console::style(guess_word.chars().nth(i).unwrap()).white()
                   }
            )
        } else {
            print!("{}", word_result[i].to_string());
        }
    }
    print!(" ");
    let color_grade = HashMap::from([
        ("G".to_string(), 4),
        ("Y".to_string(), 3),
        ("R".to_string(), 2),
        ("X".to_string(), 1)
    ]);//use hash to mark priority
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

fn guess_1(word_to_guess: &String,
           alphabet: &mut Vec<Color>,
           is_difficult: bool,
           already_guessed_position: &mut Vec<(i32, char)>) -> Result<(), Error> {
    //do guess operation once, and return updated alphabet_color
    let mut word = String::new();
    io::stdin().read_line(&mut word).expect("cannot read");
    word.pop();
    //TODO: add verification in different mode
    for i in ACCEPTABLE {
        if word == i.to_string() {
            // println!("valid!");
            // is_acceptable = true;
            return match_result(&word, word_to_guess, alphabet, is_difficult, already_guessed_position);
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