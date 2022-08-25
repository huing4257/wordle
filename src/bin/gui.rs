use fltk::{
    app,
    button::{self, Button},
    dialog,
    enums::{Align, Color, Event, Font, FrameType, Key, Shortcut},
    frame::{self, Frame},
    group::{self, Pack, PackType},
    input,
    prelude::*,
    text,
    window::{self, Window},
};
use func;
use func::{Info, RoundInfo};

pub const ALPHABET: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
pub const KeyboardAlphabet: &[char] = &[
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l',
    'z', 'x', 'c', 'v', 'b', 'n', 'm',
];
pub const Grey: u32 = 0x787c7f;
pub const Red: u32 = 0x787c7f;
pub const Green: u32 = 0x6ca965;
pub const Yellow: u32 = 0xc8b653;

#[derive(Debug, Copy, Clone)]
enum Message {
    Letter(char),
    Enter,
    Delete,
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

    let mut btn_enter = button::Button::new(50, 610, 70, 50, "@returnarrow");
    btn_enter.set_color(Color::Light2);
    let mut btn_undo = button::Button::new(510, 610, 70, 50, "@undo");
    btn_undo.set_color(Color::Light2);
    let mut char_num = 0;

    // create show frame
    let mut frame_list: Vec<Frame> = vec![];
    for i in 0..6 {
        let mut frame_pack1 = group::Pack::default_fill()
            .with_type(group::PackType::Horizontal)
            .with_pos(130, 40 + i * 75)
            .with_size(1000, 75);
        frame_pack1.auto_layout();
        for j in 0..5 {
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
        .with_type(group::PackType::Horizontal)
        .with_pos(50, 500)
        .with_size(1000, 50);
    letter_pack1.auto_layout();
    letter_pack1.set_spacing(5);
    for _i in 0..10 {
        letter_btn.push(
            button::Button::default()
                .with_size(50, 50)
                .with_label(&KeyboardAlphabet[char_num].to_string()),
        );
        char_num += 1;
    }
    letter_pack1.end();

    let mut letter_pack2 = group::Pack::default_fill()
        .with_type(group::PackType::Horizontal)
        .with_pos(80, 555)
        .with_size(1000, 50);
    letter_pack2.set_spacing(5);
    for _i in 0..9 {
        letter_btn.push(
            button::Button::default()
                .with_size(50, 50)
                .with_label(&KeyboardAlphabet[char_num].to_string()),
        );
        char_num += 1;
    }
    letter_pack2.end();

    let mut letter_pack3 = group::Pack::default_fill()
        .with_type(group::PackType::Horizontal)
        .with_pos(125, 610)
        .with_size(1000, 50);
    letter_pack3.set_spacing(5);
    for _i in 0..7 {
        letter_btn.push(
            button::Button::default()
                .with_size(50, 50)
                .with_label(&KeyboardAlphabet[char_num].to_string()),
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
    let mut word_to_guess =String::new() ;
    let mut info=Info::new();
    let mut round_info=RoundInfo::new(&info);
    let mut guess_word = String::new();
    let mut guess_time: usize = 0;
    func::get_word_by_start_day(&mut word_to_guess,&info,0);
    let (s, r) = app::channel::<Message>();

    for mut but in &mut letter_btn {
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
                    if guess_word.len() < 5 {
                        println!("ch");
                        let position = guess_time * 5 + guess_word.len();
                        frame_list[position].set_label(&ch.to_string());
                        guess_word.push(ch);
                    } else {
                        dialog::message(500, 300, "Word already full!");
                    }
                }
                Message::Enter => {
                    println!("enter");
                    reset_up_color(&mut frame_list, &mut word_to_guess, &mut guess_word);
                    app.redraw();
                }
                Message::Delete => match guess_word.pop() {
                    Some(_) => {
                        frame_list[guess_word.len()].set_label("");
                    }
                    None => dialog::message(500, 300, "Word already empty!"),
                },
            }
        }
    }
}

fn reset_up_color(frame_list: &mut Vec<Frame>, word_to_guess: &mut String, guess_word: &mut String) {
    let result = func::calculate_color(&word_to_guess, &guess_word);
    println!("{},{},{:?}", word_to_guess, guess_word, result);
    let mut is_success = true;
    for i in 0..5 {
        match result[i] {
            func::Color::R => {
                frame_list[i].set_color(Color::from_hex(Red));
            }
            func::Color::Y => {
                frame_list[i].set_color(Color::from_hex(Yellow));
                is_success = false;
            }
            func::Color::G => {
                frame_list[i].set_color(Color::from_hex(Green));
                is_success = false
            }
            func::Color::X => unreachable!(),
        }
    }
}
