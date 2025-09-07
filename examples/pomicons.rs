#![allow(unused_must_use)]

use iced::{
    Center, Element, Length, Task, font,
    widget::{column, row},
};
use iced_fonts::{POMICONS_FONT_BYTES, pomicons::*};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).title("pomicons").run()
}

#[derive(Default)]
struct App {}

#[derive(Debug, Clone, Copy)]
enum Message {
    FontLoaded(Result<(), font::Error>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {},
            Task::batch(vec![
                font::load(POMICONS_FONT_BYTES).map(Message::FontLoaded),
            ]),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FontLoaded(result) => {
                dbg!(result);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            row![
                space(),
                CLEAN_CODE(),
                POMODORO_DONE(),
                POMODORO_ESTIMATED(),
                POMODORO_TICKING(),
                POMODORO_SQUASHED(),
                SHORT_PAUSE(),
                LONG_PAUSE(),
                AWAY(),
                PAIR_PROGRAMMING(),
                INTERNAL_INTERRUPTION(),
                EXTERNAL_INTERRUPTION(),
            ]
            .padding(12)
            .spacing(20)
            .width(Length::Fill)
            .align_y(Center),
        ]
        .into()
    }
}
