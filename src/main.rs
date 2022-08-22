mod builtin_words;

use std::collections::HashMap;
use console;
use std::io::{self, Write};
use std::cmp::Ordering;
// use std::str::Chars;
use crate::builtin_words::{FINAL, ACCEPTABLE};
use rand::seq::{IteratorRandom};

pub const WORDLE_LENS: usize = 5;
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
    //initialize alphabet


    let mut word_to_guess = String::new();
    let mut mode = Mode {
        is_random: false,
        is_difficult: false,
        is_specified: false,
        is_stats: false,
        succeeded_game: 0,
        failed_game: 0,
        word_guessed_freq: vec![],
        succeed_guess_times: vec![],
        words_appeared: vec![],
    };
    //write a struct to save mode
    mode_analyze(&mut word_to_guess, &mut mode);
    let mut is_continue_playing = true;
    //play several times
    while is_continue_playing {
        match guess_whole(&mut word_to_guess, &mut mode) {
            Ok(()) => {}
            Err(err) => println!("{}", err.to_string())
        }
        //if in --word mode, break
        if mode.is_specified {
            break;
        }
        //judge if continue
        let mut choice: Option<bool> = None;
        while let None = choice {
            if is_tty {
                println!("Do you want to continue? Type [Y]or[N] to tell me");
            }
            let mut whether_another: String = String::new();
            match io::stdin().read_line(&mut whether_another) {
                Ok(n) => if n == 0 { choice = Some(false) }
                //stop when read EOF
                Err(err) => return Err(Box::from(err))
            }
            whether_another.pop();
            match &whether_another[..] {
                "Y" => choice = Some(true),
                "N" => choice = Some(false),
                _ => println!("INVALID")
            }
        }
        is_continue_playing = choice.unwrap();
        if mode.is_stats {
            let round_times = mode.succeed_guess_times.len() as i32;
            let total_times: i32 = mode.succeed_guess_times.iter().sum();
            let average= if mode.succeed_guess_times.len()!=0 {
                ( total_times / round_times) as f64
            }else { 0.00 };
            println!("{} {} {:.2}", mode.succeeded_game, mode.failed_game,average);
            mode.word_guessed_freq.sort_by(|a, b| match b.1.cmp(&a.1){
                Ordering::Greater|Ordering::Less=>b.1.cmp(&a.1),
                Ordering::Equal => a.0.cmp(&b.0)
            });
            let show_limit: i32 = if mode.word_guessed_freq.len() < 5 {
                mode.word_guessed_freq.len() as i32
            } else { 5 };
            let mut i=1;
            for temp in &mode.word_guessed_freq {
                // println!("{}",show_limit);
                // println!("{}",i);
                if  i==show_limit {
                    println!("{} {}",temp.0.to_ascii_uppercase(),temp.1);
                    break;
                }
                print!("{} {} ",temp.0.to_ascii_uppercase(),temp.1 );
                i+=1;
            }
        }
    }
    Ok(())
}

fn guess_whole(mut word_to_guess: &mut String, mut mode: &mut Mode) -> Result<(), Error> {

    let mut alphabet_color: Vec<Color> = vec![];
    for _i in 0..26 {
        let temp = Color::X;
        alphabet_color.push(temp);
    }
    let is_tty = atty::is(atty::Stream::Stdout);
    if mode.is_random {
        *word_to_guess = FINAL.iter().choose(&mut rand::thread_rng()).unwrap().to_string();
        while mode.words_appeared.contains(&word_to_guess) {
            *word_to_guess = FINAL.iter().choose(&mut rand::thread_rng()).unwrap().to_string();
        }
        mode.words_appeared.push(word_to_guess.clone());
    } else if !mode.is_specified{
        word_to_guess.clear();
        io::stdin().read_line(&mut word_to_guess).unwrap();
        word_to_guess.pop();
    }
    *word_to_guess = word_to_guess.to_ascii_lowercase();
    // println!("{}",word_to_guess);
    let mut is_success: bool = false;
    let mut guess_times = 0;
    let mut already_guessed: Vec<(i32, char)> = vec![];
    while guess_times <= 5 {
        // println!("{}",guess_times);
        //Guess 6 times
        match guess_1(&word_to_guess, &mut alphabet_color, &mut mode, &mut already_guessed) {
            Err(error) => {
                match error {
                    Error::InvalidWord => { println!("{}", error.to_string()) }
                    Error::AlreadyCorrect => {
                        guess_times += 1;
                        is_success = true;
                        print_alphabet(is_tty, &mut alphabet_color);
                        println!("{} {}", error.to_string(), guess_times);
                        mode.succeed_guess_times.push(guess_times);
                        break;
                    }
                }
            }
            Ok(_ok) => {
                guess_times += 1;
                print_alphabet(is_tty, &mut alphabet_color);
            }
        }
    }
    if !is_success {
        mode.failed_game += 1;
        if is_tty {
            println!(
                "{} {}",
                console::style("FAILED").red(),
                console::style(word_to_guess.to_ascii_uppercase()).green().italic()
            );
        } else {
            println!("FAILED {}", word_to_guess.to_ascii_uppercase());
        }
    } else {
        mode.succeeded_game += 1
    }
    return Ok(());
}

fn mode_analyze(word_to_guess: &mut String, mode: &mut Mode) {
    let mut num_args = 0;
    loop {
        //loop to analyze args
        match std::env::args().nth(num_args) {
            None => break,
            Some(arg) => {
                // println!("{arg}");
                match &arg[..] {
                    "-w" | "--word" => {
                        mode.is_specified = true;
                        *word_to_guess = std::env::args().nth(num_args + 1).expect("did not input word");
                    }
                    "-r" | "--random" => {
                        mode.is_random = true
                    }
                    "-D" | "--difficult" => {
                        mode.is_difficult = true
                    }
                    "-t" | "--stats" => {
                        mode.is_stats = true
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
                mode: &mut Mode,
                already_guessed_position: &mut Vec<(i32, char)>, ) -> Result<(), Error> {
    // Calculate the color, print a string of 5 letters, and return updated alphabet_color
    // First find G, ignore them, then match last letters one by one (first 5 letters)
    // For alphabet, use a vec of 5 to record the condition of 5 letters
    if mode.is_difficult {
        for temp in already_guessed_position.iter() {
            if guess_word.chars().nth(temp.0 as usize).unwrap() != temp.1 {
                return Err(Error::InvalidWord);
            }
        }//letters already correct cannot change
        for i in 0..26 {
            if let Color::Y = alphabet[i] {
                if !guess_word.contains(ALPHABET[i]) {
                    return Err(Error::InvalidWord);
                }
            }
        }
        //letters in wrong position must be used
    }
    //Here, the input is finally valid enough
    mode.add_guessed_word(&guess_word);
    let mut word_result: Vec<Color> = vec![];
    let mut char_to_ignore: Vec<i32> = vec![];
    for i in 0..WORDLE_LENS as i32 {
        if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(i as usize) {
            char_to_ignore.push(i);
            already_guessed_position.push((i, guess_word.chars().nth(i as usize).unwrap()))
        }
    }
    // println!("{:?}", char_to_ignore);
    let mut char_to_ignore_to_guess = char_to_ignore.clone();
    for i in 0..WORDLE_LENS as i32 {
        if char_to_ignore.contains(&i) {
            word_result.push(Color::G);
            continue;
        }
        let mut is_in: bool = false;
        for j in 0..WORDLE_LENS as i32 {
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
    for i in 0..WORDLE_LENS {
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
        for j in 0..WORDLE_LENS {
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
           mode: &mut Mode,
           already_guessed_position: &mut Vec<(i32, char)>) -> Result<(), Error> {
    //Do guess operation once, and return updated alphabet_color, if input invalid, return and try this
    //function again.
    let mut word = String::new();
    io::stdin().read_line(&mut word).expect("cannot read");
    word.pop();
    //TODO: add verification in different mode
    for i in ACCEPTABLE {
        if word == i.to_string() {
            // println!("valid!");
            // is_acceptable = true;
            return match_result(&word, word_to_guess, alphabet, mode, already_guessed_position);
        }
    }
    return Err(Error::InvalidWord);
}

struct Mode {
    is_difficult: bool,
    is_random: bool,
    is_specified: bool,
    is_stats: bool,
    succeeded_game: i32,
    failed_game: i32,
    word_guessed_freq: Vec<(String, i32)>,
    succeed_guess_times: Vec<i32>,
    words_appeared: Vec<String>,
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

impl Mode {
    fn add_guessed_word(&mut self, word: &String) {
        let mut contain = false;
        for temp in &mut self.word_guessed_freq {
            if temp.0 == *word {
                temp.1 += 1;
                contain = true
            }
        }
        if !contain {
            self.word_guessed_freq.push((word.clone(), 1))
        }
    }
}