mod builtin_words;
use func;

use std::fs;
use console;
use std::io::{self};
use serde_json;
use func::Info;




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);

    let mut word_to_guess = String::new();
    //write a struct to save info
    let mut info = Info::new();
    let mut is_continue_playing = true;
    func::info_analyze(&mut word_to_guess, &mut info).expect("args error");
    if is_tty {
        println!(
            "{}", console::style("Game Starts!").bold().blink().blue()
        );
    }
    //play several times
    while is_continue_playing {
        match func::guess_round(&mut word_to_guess, &mut info) {
            Ok(()) => {}
            Err(err) => println!("{}", err.to_string())
        }
        //if in --word info, break
        if info.is_word_specified {
            break;
        }
        if info.is_stats {
            func::print_stats(&mut info);
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

// fn recommend_from_hint_list(list:& Vec<String>){
//     let mut temp_list=list.clone();
//     temp_list.sort_by(|a,b|
//         get_grade_one_depth(list, &b).cmp(&get_grade_one_depth(list, &a))
//     );
//     let mut iter=temp_list.iter();
//         let mut count=0;
//         while let Some(t)= iter.next() {
//             print!("{} ",t);
//             count+=1;
//             if count==2 {break }
//         }
//     println!();
// }

// fn get_grade_one_depth(list: & Vec<String>, next_guess: &String) -> i32 {
//     let mut quantity_list: Vec<i32> = vec![];
//     for possible_answer in list.iter() {
//         let result = calculate_color(&possible_answer, &next_guess);
//         let acc_quantity = get_new_hint_list(&list, next_guess, &result).len() as i32;
//         quantity_list.push(acc_quantity);
//     }
//     let average: i32 = quantity_list.iter().sum();
//     average
// }


