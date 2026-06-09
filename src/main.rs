#![windows_subsystem = "windows"]
use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, column, container, pick_list, row, scrollable, slider, table, text},
};

use crate::lock::{Link, Solution};

pub mod lock;

fn main() -> Result<(), iced::Error> {
    iced::application(State::boot, State::update, State::view).run()
}

pub struct State {
    new_lock_size: u8,
    lock: Option<lock::Lock>,
    solution: Option<Solution>,
}

#[derive(Clone)]
pub enum Message {
    LockSizeChanged(u8),
    CreateLock,
    SetStart(usize, u8),
    CycleLink(usize, usize),
    Solve,
}

impl State {
    pub fn boot() -> Self {
        Self {
            new_lock_size: 4,
            lock: None,
            solution: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LockSizeChanged(size) => {
                self.new_lock_size = size;
                self.solution = None;
                Task::none()
            }
            Message::CreateLock => {
                self.lock = Some(lock::Lock::new(self.new_lock_size as usize));
                Task::none()
            }
            Message::SetStart(index, start) => {
                if let Some(lock) = &mut self.lock {
                    lock.slices[index] = start;
                }
                Task::none()
            }
            Message::CycleLink(index, linked) => {
                if let Some(lock) = &mut self.lock {
                    lock.links.cycle_link(index, linked);
                }
                Task::none()
            }
            Message::Solve => {
                if let Some(lock) = &mut self.lock {
                    self.solution = Some(lock.solve());
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        scrollable(
            column![
                container(
                    column![
                        "Lock size",
                        row![
                            text!("{}", self.new_lock_size),
                            slider(4..=10, self.new_lock_size, Message::LockSizeChanged)
                        ]
                        .spacing(5),
                        button("Create Lock").on_press(Message::CreateLock)
                    ]
                    .spacing(10)
                )
                .padding(20)
                .style(container::bordered_box),
                if let Some(lock) = &self.lock {
                    Some(
                        column![
                            container(
                                column(lock.slices.iter().enumerate().map(|(index, slice)| {
                                    container(
                                        column![
                                            text!("Slice {}", index + 1),
                                            row![
                                                text!("Start: {}", slice).width(100),
                                                slider(
                                                    1..=lock::SLICE_POSITIONS,
                                                    *slice,
                                                    move |start| {
                                                        Message::SetStart(index, start)
                                                    }
                                                ),
                                            ],
                                        ]
                                        .spacing(10),
                                    )
                                    .padding(10)
                                    .style(container::bordered_box)
                                    .into()
                                }))
                                .spacing(10),
                            )
                            .padding(20)
                            .style(container::bordered_box),
                            container(table(
                                (0..=lock.size()).into_iter().map(|index| {
                                    table::column(
                                        if index == 0 {
                                            text("")
                                        } else {
                                            text!("{}", index)
                                        },
                                        move |row: (usize, Vec<String>)| {
                                            if index == 0 {
                                                Element::from(text!("{}", row.0 + 1))
                                            } else {
                                                Element::from(
                                                    button(text!("{}", &row.1[index - 1]))
                                                        .on_press(Message::CycleLink(
                                                            row.0,
                                                            index - 1,
                                                        )),
                                                )
                                            }
                                        },
                                    )
                                }),
                                (0..lock.size()).map(|index| {
                                    let cols = lock
                                        .links
                                        .links_from(index)
                                        .iter()
                                        .map(|link| {
                                            match link {
                                                Link::None => "N",
                                                Link::Same => "S",
                                                Link::Opposite => "O",
                                            }
                                            .to_string()
                                        })
                                        .collect();
                                    (index, cols)
                                })
                            ))
                            .padding(20)
                            .style(container::bordered_box),
                            Some(button("Solve").on_press(Message::Solve)),
                        ]
                        .spacing(20),
                    )
                } else {
                    None
                },
                if let Some(solution) = &self.solution {
                    Some(
                        column![
                            "Solve Order:",
                            row(solution
                                .graph
                                .solve_order
                                .iter()
                                .map(|index| { text!("{}", index + 1).size(20).into() }))
                            .spacing(5)
                        ]
                        .spacing(10),
                    )
                } else {
                    None
                }
            ]
            .width(500)
            .padding(20)
            .spacing(20),
        )
        .width(Fill)
        .into()
    }
}
