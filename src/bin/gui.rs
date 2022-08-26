use std::fmt::Debug;
use std::fs;
use fltk::{app, button::{self, Button}, dialog, enums::{Color, Font, FrameType}, frame::{Frame},
           group::{self, PackType}, menu, prelude::*, window::{Window}};
use fltk::enums::Shortcut;
use func;
use func::{update_round_alphabet_color, Info, RoundInfo, stats_to_string};

pub const ALPHABET: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
pub const KEYBOARD_ALPHABET: &[char] = &[
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l',
    'z', 'x', 'c', 'v', 'b', 'n', 'm',
];
pub const GREY: u32 = 0xd1d1d1;
pub const RED: u32 = 0x787c7f;
pub const GREEN: u32 = 0x6ca965;
pub const YELLOW: u32 = 0xc8b653;

#[derive(Debug, Copy, Clone)]
enum Message {
    Letter(char),
    Enter,
    Delete,
    Seed,
    Save,
    Open,
    Quit,
    Show
}

fn main() {
    let app = app::App::default();
    app::set_visible_focus(false);
    app::background(0x42, 0x42, 0x42);

    let width = 635;
    let height = 700;

    let mut wind = Window::default()
        .with_label("WORDLE")
        .with_size(width, height)
        .center_screen();
    wind.set_color(Color::White);

    let (s, r) = app::channel::<Message>();

    let mut btn_enter = button::Button::new(50, 610, 70, 50, "@returnarrow");
    btn_enter.set_color(Color::Light2);
    let mut btn_undo = button::Button::new(510, 610, 70, 50, "@undo");
    btn_undo.set_color(Color::Light2);
    let mut char_num = 0;

    //create menu
    let mut menubar = menu::MenuBar::new(0, 0, width, 25, "rew");
    menubar.set_color(Color::Light3);
    menubar.set_frame(FrameType::FlatBox);
    menubar.add_emit(
        "&File/Save\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        s,
        Message::Save,
    );
    menubar.add_emit(
        "&File/Open\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        s,
        Message::Open,
    );
    menubar.add_emit(
        "&File/Show stats",
        Shortcut::Shift |'s',
        menu::MenuFlag::Normal,
        s,
        Message::Show
    );
    menubar.add_emit(
        "&File/Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        s,
        Message::Quit,
    );
    menubar.add_emit(
        "&Settings/Seed...\t",
        Shortcut::Shift | 's',
        menu::MenuFlag::Normal,
        s,
        Message::Seed,
    );



    // create show frame
    let mut frame_list: Vec<Frame> = vec![];
    for i in 0..6 {
        let mut frame_pack1 = group::Pack::default_fill()
            .with_type(PackType::Horizontal)
            .with_pos(130, 40 + i * 75)
            .with_size(1000, 75);
        frame_pack1.auto_layout();
        for _j in 0..5 {
            let mut temp = Frame::default().with_size(75, 75);
            temp.set_frame(FrameType::DownBox);
            temp.set_color(Color::Light3);
            frame_list.push(temp)
        }
        frame_pack1.end();
    }

    //create letter button
    let mut letter_btn: Vec<Button> = vec![];

    let mut letter_pack1 = group::Pack::default_fill()
        .with_type(PackType::Horizontal)
        .with_pos(50, 500)
        .with_size(1000, 50);
    letter_pack1.auto_layout();
    letter_pack1.set_spacing(5);
    for _i in 0..10 {
        letter_btn.push(
            Button::default()
                .with_size(50, 50)
                .with_label(&KEYBOARD_ALPHABET[char_num].to_string().to_ascii_uppercase()),
        );
        char_num += 1;
    }
    letter_pack1.end();

    let mut letter_pack2 = group::Pack::default_fill()
        .with_type(PackType::Horizontal)
        .with_pos(80, 555)
        .with_size(1000, 50);
    letter_pack2.set_spacing(5);
    for _i in 0..9 {
        letter_btn.push(
            Button::default()
                .with_size(50, 50)
                .with_label(&KEYBOARD_ALPHABET[char_num].to_string().to_ascii_uppercase()),
        );
        char_num += 1;
    }
    letter_pack2.end();

    let mut letter_pack3 = group::Pack::default_fill()
        .with_type(PackType::Horizontal)
        .with_pos(125, 610)
        .with_size(1000, 50);
    letter_pack3.set_spacing(5);
    for _i in 0..7 {
        letter_btn.push(
            Button::default()
                .with_size(50, 50)
                .with_label(&KEYBOARD_ALPHABET[char_num].to_string().to_ascii_uppercase()),
        );
        char_num += 1;
    }
    letter_pack3.end();
    //set button style
    for i in &mut letter_btn {
        i.set_color(Color::Light3);
        i.set_label_font(Font::Helvetica)
    }

    //start game
    let mut word_to_guess = String::new();
    let mut info = Info::new();
    let mut round_info = RoundInfo::new(&info);
    let mut guess_word = String::new();
    let mut guess_count: usize = 0;
    let mut is_good: bool = true;// a condition variable controlled by return
    let mut args: Vec<String> = vec![];
    func::get_word_by_start_day(&mut word_to_guess, &info, 0);
    let mut is_success = false;
    for but in &mut letter_btn {
        but.emit(s, Message::Letter(but.label().chars().next().unwrap()))
    }
    btn_enter.emit(s, Message::Enter);
    btn_undo.emit(s, Message::Delete);
    wind.end();
    wind.show();

    while app.wait() {
        if let Some(val) = r.recv() {
            match val {
                Message::Letter(ch) => {
                    if is_good {
                        if guess_word.len() < 5 {
                            println!("ch");
                            let position = guess_count * 5 + guess_word.len();
                            frame_list[position].set_label(&ch.to_string());
                            guess_word.push(ch.to_ascii_lowercase());
                        } else {
                            dialog::message(500, 300, "Word already full!");
                        }
                    } else {
                        dialog::message(500, 300, "Please click return arrow to reset!");
                    }
                }
                Message::Enter => {
                    println!("enter");

                    if is_good {
                        if info.acceptable_set.contains(&guess_word) {
                            let result = func::calculate_color(&word_to_guess, &guess_word);
                            is_success = true;
                            update_round_alphabet_color(&mut round_info, &guess_word, &result);
                            for i in 0..5 {
                                let color = result[i].to_hex();
                                frame_list[guess_count * 5 + i].set_color(Color::from_hex(color));
                                //judge if succeed
                                if let func::Color::G = result[i] {} else {
                                    is_success = false;
                                }
                            }
                            for i in 0..26 {
                                for num_in_alpha in 0..26 {
                                    if letter_btn[i].label().chars().next().unwrap().to_ascii_lowercase()
                                        == ALPHABET[num_in_alpha]
                                    {
                                        let color = round_info.alphabet_color[num_in_alpha].to_hex();
                                        letter_btn[i].set_color(Color::from_hex(color));
                                    }
                                }
                            }
                            round_info.word_guessed_this_round.push(guess_word.clone().to_ascii_uppercase());
                            app.redraw();
                            //new guess
                            guess_word.clear();
                            guess_count += 1;
                            if guess_count == 6 {
                                dialog::message(500, 300, "You failed.Click return arrow to reset.");
                                is_good = false;
                            }
                        } else {
                            dialog::message(500, 300, "Word doesn't exist!");
                        }
                        if is_success {
                            info.state.games.push(func::Game { answer: word_to_guess.clone(), guesses: round_info.word_guessed_this_round.clone() });
                            dialog::message(500, 300, "You win! Click return arrow to reset ");
                            is_good = false;
                        }
                    } else {
                        let choices = dialog::choice2(
                            500, 300, "Do you want to start a new round?",
                            "Yes", "Cancel", "");
                        match choices {
                            Some(choice) => {
                                match choice {
                                    0 => {
                                        is_good = true;
                                        is_success = false;
                                        round_info = RoundInfo::new(&info);
                                        for frame in &mut frame_list {
                                            frame.set_color(Color::from_hex(GREY));
                                            frame.set_label("")
                                        }
                                        for btn in &mut letter_btn {
                                            btn.set_color(Color::from_hex(GREY));
                                            btn.redraw();
                                        }
                                        //update round info
                                        info.state.games.push(func::Game {
                                            answer: word_to_guess.clone().to_ascii_uppercase(),
                                            guesses: round_info.word_guessed_this_round.clone(),
                                        });
                                        info.state.total_rounds += 1;
                                        guess_count = 0;
                                        info.day += 1;
                                        word_to_guess =
                                            info.final_set[info.shuffled_seq[info.day as usize]].clone();
                                    }
                                    1 => {
                                        is_good = false;
                                    }
                                    _ => unreachable!()
                                }
                            }
                            None => { is_good = false }
                        }
                    }
                }
                Message::Delete => {
                    if is_good {
                        match guess_word.pop() {
                            Some(_) => {
                                frame_list[guess_count * 5 + guess_word.len()].set_label("");
                            }
                            None => dialog::message(500, 300, "Word already empty!"),
                        }
                    } else {
                        dialog::message(500, 300, "You have won! Please click return arrow.");
                    }
                }
                Message::Seed => {
                    let seed = dialog::input(
                        500, 300, "Start game with a random seed:", "");
                    args.push("--seed".to_string());
                    if let Some(s) = seed {
                        args.push(s);
                        match func::info_analyze(&mut word_to_guess, &mut info, &args) {
                            Ok(_) => {
                                round_info = RoundInfo::new(&info);
                                func::get_word_by_start_day(&mut word_to_guess, &info, 0);
                            }
                            Err(err) => {
                                dialog::message(500, 300, &err.to_string());
                            }
                        }
                    }
                }
                Message::Save => {
                    let mut saving = dialog::NativeFileChooser::new(
                        dialog::NativeFileChooserType::BrowseSaveFile);
                    saving.show();
                    info.state_path = saving.filename().into_os_string().into_string().unwrap();
                    let state_string = serde_json::to_string_pretty(&info.state).unwrap();
                    fs::write(saving.filename(), state_string).unwrap();
                }
                Message::Open => {
                    let mut saving = dialog::NativeFileChooser::new(
                        dialog::NativeFileChooserType::BrowseFile);
                    saving.show();
                    let state_path = saving.filename().into_os_string().into_string().unwrap();
                    args.push("-S".to_string());
                    args.push(state_path);
                    func::info_analyze(&mut word_to_guess, &mut info, &args).expect("input error");
                }
                Message::Quit => {
                    if info.state_path.is_empty() {
                        if let Some(i) = dialog::choice2_default(
                            "Didn't save, sure to quit?", "No", "yes", "") {
                            match i {
                                1 => app.quit(),
                                _ => {}
                            }
                        }
                    }
                }
                Message::Show => {
                    println!("{}",stats_to_string(&mut info));
                    dialog::message(500, 300,&stats_to_string(&mut info))
                }
            }
        }
    }
}
