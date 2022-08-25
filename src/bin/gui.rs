use fltk::{
    app,
    button::{self, Button},
    enums::{Align, Color, Event, Font, FrameType, Key, Shortcut},
    frame::{self, Frame},
    group::{self, Pack, PackType},
    input,
    prelude::*,
    text,
    window::{self, Window},
};
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
                .with_label(&KeyboardAlphabet[char_num].to_ascii_uppercase().to_string()),
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
                .with_label(&KeyboardAlphabet[char_num].to_ascii_uppercase().to_string()),
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
                .with_label(&KeyboardAlphabet[char_num].to_ascii_uppercase().to_string()),
        );
        char_num += 1;
    }
    letter_pack3.end();

    for i in &mut letter_btn {
        i.set_color(Color::Light3);
        i.set_label_font(Font::Helvetica)
    }
    btn_undo.set_callback(|_| println!("The button was clicked!"));
    wind.end();
    wind.show();

    app.run().unwrap();
}
