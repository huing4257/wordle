mod builtin_words;

use std::fs;
use std::collections::HashMap;
use console;
use std::io::{self};
use std::cmp::Ordering;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use serde_json;
// use std::str::Chars;
use crate::builtin_words::{FINAL, ACCEPTABLE};

pub const WORDLE_LENS: usize = 5;
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
    let mut info = Info {
        is_random: false,
        is_difficult: false,
        is_word_specified: false,
        is_stats: false,
        is_seeded: false,
        is_special_day: false,
        is_stated: false,
        succeeded_game: 0,
        failed_game: 0,
        words_appeared: vec![],
        day: 1,
        seed: 0,
        shuffled_seq: {
            let mut temp: Vec<usize> = (0..FINAL.len()).collect();
            let mut rng: StdRng = SeedableRng::seed_from_u64(0);
            temp.shuffle(&mut rng);
            temp
        },
        final_path: "".to_string(),
        acceptable_path: "".to_string(),
        final_set: {
            let mut a: Vec<String> = vec![];
            for i in FINAL {
                a.push(i.to_string());
            };
            a
        },
        acceptable_set: {
            let mut a: Vec<String> = vec![];
            for i in ACCEPTABLE {
                a.push(i.to_string());
            };
            a
        },
        state: {
            State {
                total_rounds: 0,
                games: vec![],
            }
        },
        state_path: String::new(),
    };
    //write a struct to save info
    info_analyze(&mut word_to_guess, &mut info).expect("args error");
    let mut is_continue_playing = true;
    //play several times
    while is_continue_playing {
        match guess_whole(&mut word_to_guess, &mut info) {
            Ok(()) => {}
            Err(err) => println!("{}", err.to_string())
        }
        //if in --word info, break
        if info.is_word_specified {
            break;
        }
        if info.is_stats {
            print_stats(&mut info);
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
                "" => choice = Some(false),
                _ => {
                    println!("INVALID")
                }
            }
        }
        is_continue_playing = choice.unwrap();
    }
    //update state file
    if info.is_stated && info.is_random {
        let state_string = serde_json::to_string_pretty(&info.state).unwrap();
        fs::write(info.state_path, state_string).unwrap();
    }
    Ok(())
}

/// Receive info, print statistics of guesses
fn print_stats(info: &mut Info) {
    let mut succeed_rounds: f64 = 0.0;
    let mut succeed_total_guess_times: f64 = 0.0;
    let mut word_guessed_freq: Vec<(String, i32)> = vec![];

    // println!("{}", serde_json::to_string_pretty(&info.state).unwrap());
    for temp in &info.state.games {
        if temp.guesses.contains(&temp.answer) {
            succeed_rounds += 1.0;
            succeed_total_guess_times += temp.guesses.len() as f64;
        }
        for guess in &temp.guesses {
            add_guessed_word(&mut word_guessed_freq, &guess);
        }
    }
    let average = if succeed_rounds != 0.0 {
        succeed_total_guess_times / succeed_rounds
    } else { 0.00 };
    println!("{:.0} {} {:.2}", succeed_rounds, info.state.total_rounds - succeed_rounds as i32, average);
    word_guessed_freq.sort_by(|a, b| match b.1.cmp(&a.1) {
        Ordering::Greater | Ordering::Less => b.1.cmp(&a.1),
        Ordering::Equal => a.0.cmp(&b.0)
    });
    let show_limit: i32 = if word_guessed_freq.len() < 5 {
        word_guessed_freq.len() as i32
    } else { 5 };
    let mut i = 1;
    for temp in &word_guessed_freq {
        // println!("{}",show_limit);
        // println!("{}",i);
        if i == show_limit {
            println!("{} {}", temp.0.to_ascii_uppercase(), temp.1);
            break;
        }
        print!("{} {} ", temp.0.to_ascii_uppercase(), temp.1);
        i += 1;
    }
}

/// Analyze args to change info
/// Return a result with Error, invalid input or args
fn info_analyze(word_to_guess: &mut String, info: &mut Info) -> Result<(), Error> {
    let mut num_args = 0;
    //loop to analyze args
    //first load config
    loop {
        match std::env::args().nth(num_args) {
            //first decide sets
            None => break,
            Some(arg) => {
                match &arg[..] {
                    "-c" | "--config" => {
                        let config_path = std::env::args().nth(num_args + 1).expect("did not input word");
                        let config_string = fs::read_to_string(&config_path).expect("config file error");
                        let config: serde_json::Value = serde_json::from_str(&config_string).expect("config file error");
                        // println!("TEST1");
                        info.load_config(word_to_guess, &config);
                        // println!("TEST2");
                        // println!("{}",serde_json::to_string_pretty(&config).unwrap());
                    }
                    _ => {}
                }
            }
        }
        num_args += 1;
    }
    //next decide sets
    let mut num_args = 0;
    loop {
        match std::env::args().nth(num_args) {
            //first decide sets
            None => break,
            Some(arg) => {
                match &arg[..] {
                    "-f" | "--final-set" => {
                        info.final_path = std::env::args().nth(num_args + 1).expect("did not input word");
                    }
                    "-a" | "--acceptable-set" => {
                        info.acceptable_path = std::env::args().nth(num_args + 1).expect("did not input word");
                    }
                    _ => {}
                }
            }
        }
        num_args += 1;
    }
    // println!("TEST point2");
    if !info.final_path.is_empty() {
        set_from_path(&info.final_path, &mut info.final_set);
    }
    if !info.acceptable_path.is_empty() {
        set_from_path(&info.acceptable_path, &mut info.acceptable_set);
    }
    //verify specified sets' contain relationship
    let mut is_contain: bool = true;
    for temp in &info.final_set {
        if !info.acceptable_set.contains(&temp) {
            is_contain = false;
        }
    }
    if !is_contain {
        return Err(Error::InvalidArgs);
    }
    // println!("ser valid");
    let mut num_args = 0;
    loop {
        //loop to analyze args
        match std::env::args().nth(num_args) {
            //first decide sets
            None => break,
            Some(arg) => {
                // println!("{arg}");

                match &arg[..] {
                    "-w" | "--word" => {
                        info.is_word_specified = true;
                        *word_to_guess = std::env::args().nth(num_args + 1).expect("did not input word");
                    }
                    "-r" | "--random" => {
                        info.is_random = true
                    }
                    "-D" | "--difficult" => {
                        info.is_difficult = true
                    }
                    "-t" | "--stats" => {
                        info.is_stats = true
                    }
                    "-d" | "--day" => {
                        info.is_special_day = true;
                        info.day =
                            std::env::args().nth(num_args + 1).expect("did not input day").parse().unwrap();
                    }
                    "-s" | "--seed" => {
                        info.is_seeded = true;
                        info.seed =
                            std::env::args().nth(num_args + 1).expect("did not input seed").parse().unwrap();
                    }
                    "-S" | "--state" => {
                        info.is_stated = true;
                        info.state_path = std::env::args().nth(num_args + 1).expect("did not input word");
                        if let Ok(state_string) = fs::read_to_string(&info.state_path) {
                            // println!("{}", info.state_path);
                            // let mut s = String::new();
                            // io::stdin().read_line(&mut s).unwrap();
                            if state_string != "{}" {
                                info.state = match serde_json::from_str(&state_string) {
                                    Ok(x) => x,
                                    Err(_) => {
                                        println!("cannot match");
                                        return Err(Error::InvalidArgs);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        num_args += 1;
    }

    if info.is_seeded {
        info.shuffled_seq = {
            let mut temp: Vec<usize> = (0..info.final_set.len()).collect();
            let mut rng: StdRng = SeedableRng::seed_from_u64(info.seed);
            temp.shuffle(&mut rng);
            temp
        }
    }
    //deal with conflict args
    if info.is_random {
        if info.is_word_specified {
            return Err(Error::InvalidArgs);
        }
    }
    if info.is_word_specified {
        if info.is_seeded && info.is_special_day {
            return Err(Error::InvalidArgs);
        }
    }
    Ok(())
}

/// Receives the word to guess this round, then starts a round of game
/// Return a result with Error
fn guess_whole(mut word_to_guess: &mut String, mut info: &mut Info) -> Result<(), Error> {
    let is_tty = atty::is(atty::Stream::Stdout);
    let game_time = info.failed_game + info.succeeded_game;
    let mut is_success: bool = false;
    let mut guess_times = 0;
    let mut round_info = RoundInfo {
        already_guessed_position: vec![],
        alphabet_color: vec![],
        word_guessed_this_round: vec![],
    };for _i in 0..26 {
        let temp = Color::X;
        round_info.alphabet_color.push(temp);
    }
    if info.is_random {
        let start_day = game_time + info.day - 1;//cause do not exist day0
        loop {
            *word_to_guess = info.final_set.iter().nth(info.shuffled_seq[start_day as usize]).unwrap().to_string();
            if !info.words_appeared.contains(&word_to_guess) {
                break;
            } else {
                *word_to_guess = info.final_set.iter().nth(info.shuffled_seq[start_day as usize]).unwrap().to_string();
            }
        }
        info.words_appeared.push(word_to_guess.clone());
    } else if !info.is_word_specified {
        word_to_guess.clear();
        io::stdin().read_line(&mut word_to_guess).unwrap();
        word_to_guess.pop();
    }
    *word_to_guess = word_to_guess.to_ascii_lowercase();
    // println!("{}",word_to_guess);
    while guess_times <= 5 {
        //Guess 6 times
        match guess_1( word_to_guess, &mut info, &mut round_info) {
            Err(error) => {
                match error {
                    Error::InvalidWord => { println!("{}", error.to_string()) }
                    Error::AlreadyCorrect => {
                        guess_times += 1;
                        is_success = true;
                        print_alphabet(&mut round_info.alphabet_color);
                        println!("{} {}", error.to_string(), guess_times);
                        break;
                    }
                    Error::InvalidArgs => {}
                }
            }
            Ok(_) => {
                guess_times += 1;
                print_alphabet(&mut round_info.alphabet_color);
            }
        }
    }
    // println!("{:?}",word_guessed_this_round);
    info.state.games.push(Game { answer: word_to_guess.clone().to_ascii_uppercase(), guesses: round_info.word_guessed_this_round });
    info.state.total_rounds += 1;

    if !is_success {
        info.failed_game += 1;
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
        info.succeeded_game += 1
    }
    return Ok(());
}


/// Receives a vector of 26 color strings and print
/// If in tty, print letters, else just print color
fn print_alphabet(alphabet_color: &Vec<Color>) {
    let is_tty = atty::is(atty::Stream::Stdout);
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
                word_to_guess:& mut String,
                info: &mut Info,
                round_info: &mut RoundInfo) -> Result<(), Error> {
    // Calculate the color, print a string of 5 letters, and return updated alphabet_color
    // First find G, ignore them, then match last letters one by one (first 5 letters)
    // For alphabet, use a vec of 5 to record the condition of 5 letters
    if info.is_difficult {
        for temp in round_info.already_guessed_position.iter() {
            if guess_word.chars().nth(temp.0 as usize).unwrap() != temp.1 {
                return Err(Error::InvalidWord);
            }
        }//letters already correct cannot change
        for i in 0..26 {
            if let Color::Y = round_info.alphabet_color[i] {
                if !guess_word.contains(ALPHABET[i]) {
                    return Err(Error::InvalidWord);
                }
            }
        }
        //letters in wrong position must be used
    }
    //Here, the input is finally valid enough
    round_info.word_guessed_this_round.push(guess_word.clone().to_ascii_uppercase());
    let mut word_result: Vec<Color> = vec![];
    let mut char_to_ignore: Vec<i32> = vec![];
    for i in 0..WORDLE_LENS as i32 {
        if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(i as usize) {
            char_to_ignore.push(i);
            round_info.already_guessed_position.push((i, guess_word.chars().nth(i as usize).unwrap()))
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
        let is_tty = atty::is(atty::Stream::Stdout);
        if is_tty {
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
                    &round_info.alphabet_color[i].to_string()) < color_grade.get(&word_result[j].to_string()
                ) {
                    round_info.alphabet_color[i] = word_result[j].clone();
                }
            }
        }
    }
    if guess_word == word_to_guess { return Err(Error::AlreadyCorrect); }
    return Ok(());
}

fn guess_1(
    word_to_guess:& mut String,
    info: &mut Info,
    round_info: &mut RoundInfo,
) -> Result<(), Error> {
    //Do guess operation once, and return updated alphabet_color, if input invalid, return and try this
    //function again.
    let mut word = String::new();
    io::stdin().read_line(&mut word).expect("cannot read");
    word.pop();
    for i in &info.acceptable_set {
        if word == i.to_string() {
            return match_result(&word, word_to_guess, info, round_info);
        }
    }

    return Err(Error::InvalidWord);
}

#[derive(Serialize, Deserialize, Debug)]
struct Game {
    answer: String,
    guesses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    total_rounds: i32,
    games: Vec<Game>,
}

struct Info {
    is_difficult: bool,
    is_random: bool,
    is_word_specified: bool,
    is_stats: bool,
    is_seeded: bool,
    is_special_day: bool,
    is_stated: bool,
    succeeded_game: i32,
    failed_game: i32,
    words_appeared: Vec<String>,
    day: i32,
    seed: u64,
    shuffled_seq: Vec<usize>,
    final_path: String,
    acceptable_path: String,
    final_set: Vec<String>,
    acceptable_set: Vec<String>,
    state: State,
    state_path: String,
}

struct RoundInfo {
    already_guessed_position: Vec<(i32, char)>,
    alphabet_color: Vec<Color>,
    word_guessed_this_round: Vec<String>
}


impl Info {
    fn load_config(&mut self, word_to_guess: &mut String, config: &serde_json::Value) {
        if let Some(is_random) = config.get("random") {
            self.is_random = is_random.as_bool().expect("config file error");
        }
        if let Some(is_difficult) = config.get("difficult") {
            self.is_difficult = is_difficult.as_bool().expect("config file error");
        }
        if let Some(is_stats) = config.get("stats") {
            self.is_stats = is_stats.as_bool().expect("config file error");
        }
        if let Some(day) = config.get("day") {
            self.is_special_day = true;
            self.day = day.as_i64().expect("config file error") as i32;
        }
        if let Some(seed) = config.get("seed") {
            self.is_seeded = true;
            self.seed = seed.as_u64().expect("config file error");
        }
        if let Some(final_set_path) = config.get("final_set") {
            self.final_path = final_set_path.as_str().expect("config file error").to_string();
        }
        if let Some(acceptable_path) = config.get("acceptable_set") {
            self.acceptable_path = acceptable_path.as_str().expect("config file error").to_string();
        }
        if let Some(state_path) = config.get("state") {
            self.is_stated = true;
            self.state_path = state_path.as_str().expect("config file error").to_string();
            if let Ok(state_string) = fs::read_to_string(&self.state_path) {
                if state_string != "{}" {
                    self.state = serde_json::from_str(&state_string).expect("config file error")
                }
            }
        }
        if let Some(word) = config.get("word") {
            self.is_word_specified = true;
            *word_to_guess = word.as_str().expect("config file error").to_string();
        }
    }
}

#[derive(Debug)]
enum Error {
    InvalidWord,
    AlreadyCorrect,
    InvalidArgs,
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
            Error::InvalidArgs => { "InvalidArgs".to_string() }
        };
    }
}

fn add_guessed_word(freq_list: &mut Vec<(String, i32)>, word: &String) {
    let mut contain = false;
    for temp in freq_list.iter_mut() {
        if temp.0 == *word {
            temp.1 += 1;
            contain = true
        }
    }
    if !contain {
        freq_list.push((word.clone(), 1))
    }
}

fn set_from_path(path: &String, set: &mut Vec<String>) {
    if let Ok(whole_string) = fs::read_to_string(&path) {
        set.clear();
        for temp in whole_string.split_terminator("\n")
        {
            set.push(temp.to_string().to_ascii_lowercase());
        }
        set.sort();
    }
}