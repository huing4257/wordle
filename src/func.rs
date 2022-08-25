use std::collections::HashMap;
use rand::prelude::{SliceRandom, StdRng};
use rand::SeedableRng;
use std::{fs, io};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use crate::builtin_words::{ACCEPTABLE, FINAL};
pub const WORDLE_LENS: usize = 5;
pub const ALPHABET: &[char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
    'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

/// Analyze args to change info
/// Return a result with Error, invalid input or args
pub fn info_analyze(word_to_guess: &mut String, info: &mut Info) -> Result<(), Error> {
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
                        assert!(FINAL.contains(&&word_to_guess[..]),"Input illegal! ");
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
                    "-h" | "--hint" => {
                        info.is_hint = true;
                    }
                    "-u" | "--tui"=>{info.is_tui=true}
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
pub fn guess_round(mut word_to_guess: &mut String, mut info: &mut Info) -> Result<(), Error> {
    let is_tty = atty::is(atty::Stream::Stdout);
    let game_time = info.failed_game + info.succeeded_game;
    let mut is_success: bool = false;
    let mut guess_times = 0;
    let mut round_info = RoundInfo {
        already_guessed_position: vec![],
        alphabet_color: vec![],
        word_guessed_this_round: vec![],
        hint_list: info.acceptable_set.clone(),
    };
    if is_tty {
        println!("This is round {}, please input your guesses",
                 console::style(info.state.total_rounds+1).green().bold());
    }
    //initialize alphabet of color
    for _i in 0..26 {
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
        assert!(FINAL.contains(&&word_to_guess[..]),"Input illegal! ");
    }
    *word_to_guess = word_to_guess.to_ascii_lowercase();
    // println!("{}",word_to_guess);
    while guess_times <= 5 {
        //Guess 6 times

        match guess_one_time(word_to_guess, &mut info, &mut round_info) {
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
                console::style("FAILED").red().bold(),
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

/// Take word to guess, info and round info, do guess once.
/// Calculate the color, print a string of 5 letters, update info
/// First find G, ignore them, then match last letters one by one (first 5 letters)
/// For alphabet, use a vec of 5 to record the condition of 5 letters
pub fn guess_one_time(
    word_to_guess: &mut String,
    info: &mut Info,
    round_info: &mut RoundInfo) -> Result<(), Error> {
    let guess_word = get_checked_guess(&info, round_info)?;
    //Here, the input is finally valid enough
    round_info.word_guessed_this_round.push(guess_word.clone().to_ascii_uppercase());

    let word_result = calculate_color(word_to_guess,  &guess_word);
    for i in 0..WORDLE_LENS {
        if let Color::G=word_result[i]{
            round_info.already_guessed_position.push(
                (i as i32, guess_word.chars().nth(i as usize).unwrap())
            )
        }
    }
    if info.is_hint {
        round_info.hint_list=get_new_hint_list(&mut round_info.hint_list, &guess_word, & word_result);
        println!("total:{}\n{:?}",round_info.hint_list.len(),round_info.hint_list);
        // recommend_from_hint_list(&mut round_info.hint_list);
    }
    //print the match result
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
    //use hash to mark priority
    let color_grade = HashMap::from([
        ("G".to_string(), 4),
        ("Y".to_string(), 3),
        ("R".to_string(), 2),
        ("X".to_string(), 1)
    ]);
    //update alphabet_color
    for char_num in 0..26 {
        for position_in_guess in 0..WORDLE_LENS {
            if ALPHABET[char_num] == guess_word.chars().nth(position_in_guess).unwrap() {
                if color_grade.get(
                    &round_info.alphabet_color[char_num].to_string())
                    <
                    color_grade.get(&word_result[position_in_guess].to_string()
                    ) {
                    round_info.alphabet_color[char_num] = word_result[position_in_guess].clone();
                }
            }
        }
    }
    if guess_word == *word_to_guess { return Err(Error::AlreadyCorrect); }
    return Ok(());
}

/// Receive two words, and give their match degree in form of color vector
pub fn calculate_color(word_to_guess: & String,  guess_word: &String) -> Vec<Color> {
    let mut word_result: Vec<Color> = vec![];
    let mut correct_position_this_round: Vec<i32> = vec![];
    for i in 0..WORDLE_LENS as i32 {
        if guess_word.chars().nth(i as usize) == word_to_guess.chars().nth(i as usize) {
            correct_position_this_round.push(i);
        }
    }
    let mut char_to_ignore_to_guess = correct_position_this_round.clone();

    //calculate word_result
    for position_in_guess in 0..WORDLE_LENS as i32 {
        if correct_position_this_round.contains(&position_in_guess) {
            word_result.push(Color::G);
            continue;
        }
        let mut is_in: bool = false;
        for position_in_answer in 0..WORDLE_LENS as i32 {
            //skip correct position
            if char_to_ignore_to_guess.contains(&position_in_answer) {
                continue;
            }
            //mark which letter in goal is appeared in wrong place, make sure G + Y <= actual num
            if guess_word.chars().nth(position_in_guess as usize) == word_to_guess.chars().nth(position_in_answer as usize) {
                word_result.push(Color::Y);
                char_to_ignore_to_guess.push(position_in_answer);
                is_in = true;
                break;
            }
        }
        if !is_in {
            word_result.push(Color::R);
        }
    }
    word_result
}

/// Receive info, print statistics of guesses
pub fn print_stats(info: &mut Info) {
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
            add_word_to_freq_list(&mut word_guessed_freq, &guess);
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

/// Receives a vector of 26 color strings and print
/// If in tty, print letters, else just print color
pub fn print_alphabet(alphabet_color: &Vec<Color>) {
    let is_tty = atty::is(atty::Stream::Stdout);
    if is_tty {
        for i in 0..26 {
            print!("{}",
                   match alphabet_color[i] {
                       Color::Y => console::style(ALPHABET[i].to_ascii_uppercase()).yellow().bold(),
                       Color::R => console::style(ALPHABET[i].to_ascii_uppercase()).white().bold(),
                       Color::G => console::style(ALPHABET[i].to_ascii_uppercase()).green().bold(),
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

pub fn add_word_to_freq_list(freq_list: &mut Vec<(String, i32)>, word: &String) {
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

/// Receive a string of path, update a set of word form the file in the path
pub fn set_from_path(path: &String, set: &mut Vec<String>) {
    if let Ok(whole_string) = fs::read_to_string(&path) {
        set.clear();
        for temp in whole_string.split_terminator("\n")
        {
            set.push(temp.to_string().to_ascii_lowercase());
        }
        set.sort();
    }
}

pub fn color_vec_to_string(vec:&Vec<Color>) ->String{
    let mut str=String::new();
    for i in vec {
        str.push(i.to_string().parse().unwrap());
    }
    str
}

///Receive a word list, a word guessed, and a guess result, return a word list contains all words in the former list
///that matches the result
pub fn get_new_hint_list(hint_list: & Vec<String>, guess_word: &String, word_result: & Vec<Color>) ->Vec<String>{
    let mut new_hint: Vec<String> = vec![];
    for acc in hint_list {
        if color_vec_to_string(&calculate_color(acc, guess_word)) ==
            color_vec_to_string(word_result) {
            new_hint.push(acc.clone());
        }
    }
    new_hint
}

pub fn get_checked_guess(info: &&mut Info, round_info: &mut RoundInfo) -> Result<String, Error> {
    let mut guess_word = String::new();
    let mut is_in_acc = false;
    io::stdin().read_line(&mut guess_word).expect("cannot read");
    guess_word.pop();
    for acceptable in &info.acceptable_set {
        if guess_word == acceptable.to_string() {
            is_in_acc = true;
        }
    }
    if !is_in_acc {
        return Err(Error::InvalidWord);
    }
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
    Ok(guess_word)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    answer: String,
    guesses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    total_rounds: i32,
    games: Vec<Game>,
}

pub struct Info {
    is_difficult: bool,
    pub is_random: bool,
    pub is_word_specified: bool,
    pub is_stats: bool,
    is_tui:bool,
    is_seeded: bool,
    is_special_day: bool,
    pub is_stated: bool,
    is_hint: bool,
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
    pub state: State,
    pub state_path: String,
}

pub struct RoundInfo {
    already_guessed_position: Vec<(i32, char)>,
    alphabet_color: Vec<Color>,
    word_guessed_this_round: Vec<String>,
    hint_list: Vec<String>,
}


impl Info {
    pub fn new() ->Info{
        Info {
            is_random: false,
            is_difficult: false,
            is_word_specified: false,
            is_stats: false,
            is_tui: false,
            is_seeded: false,
            is_special_day: false,
            is_stated: false,
            is_hint: false,
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
                for i in FINAL.iter() {
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
        }
    }
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
pub enum Error {
    InvalidWord,
    AlreadyCorrect,
    InvalidArgs,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Color {
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
    pub fn to_string(&self) -> String {
        return match &self {
            Error::InvalidWord => { "INVALID".to_string() }
            Error::AlreadyCorrect => { "CORRECT".to_string() }
            Error::InvalidArgs => { "InvalidArgs".to_string() }
        };
    }
}
