use iced::{
    Element, Task,
    widget::{button, column, container, pick_list, row, slider, text},
};

use crate::lock::Direction;

pub mod lock;

fn main() -> Result<(), iced::Error> {
    iced::application(State::boot, State::update, State::view).run()
}

pub struct State {
    new_lock_size: u8,
    lock: Option<lock::Lock>,
}

#[derive(Clone)]
pub enum Message {
    LockSizeChanged(u8),
    CreateLock,
    SetStart(usize, u8),
    SetTarget(usize, u8),
    SetLink(usize, usize, Direction),
    RemoveLink(usize, usize),
}

impl State {
    pub fn boot() -> Self {
        Self {
            new_lock_size: 4,
            lock: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LockSizeChanged(size) => {
                self.new_lock_size = size;
                Task::none()
            }
            Message::CreateLock => {
                self.lock = Some(lock::Lock::new(self.new_lock_size as usize));
                Task::none()
            }
            Message::SetStart(index, start) => {
                if let Some(lock) = &mut self.lock {
                    lock.slices[index].start = start;
                }
                Task::none()
            }
            Message::SetTarget(index, target) => {
                if let Some(lock) = &mut self.lock {
                    lock.slices[index].target = target;
                }
                Task::none()
            }
            Message::SetLink(index, linked, direction) => {
                if let Some(lock) = &mut self.lock {
                    lock.slices[index].linked.insert(linked, direction);
                }
                Task::none()
            }
            Message::RemoveLink(index, linked) => {
                if let Some(lock) = &mut self.lock {
                    lock.slices[index].linked.remove(&linked);
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
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
                    container(
                        column(lock.slices.iter().enumerate().map(|(index, slice)| {
                            container(
                                column![
                                    text!("Slice {}", index + 1),
                                    row![
                                        text!("Start: {}", slice.start).width(100),
                                        slider(
                                            1..=lock::SLICE_POSITIONS,
                                            slice.start,
                                            move |start| { Message::SetStart(index, start) }
                                        ),
                                    ],
                                    row![
                                        text!("Target: {}", slice.target).width(100),
                                        slider(
                                            1..=lock::SLICE_POSITIONS,
                                            slice.target,
                                            move |target| { Message::SetTarget(index, target) }
                                        ),
                                    ],
                                    "Linked:",
                                    column(slice.linked.iter().map(|(linked, direction)| {
                                        row![
                                            text!("{}", linked),
                                            pick_list(
                                                [Direction::Same, Direction::Opposite],
                                                Some(direction),
                                                move |direction| Message::SetLink(
                                                    index, *linked, direction
                                                )
                                            ),
                                            button("X")
                                                .on_press(Message::RemoveLink(index, *linked))
                                        ]
                                        .spacing(10)
                                        .into()
                                    }))
                                    .spacing(10),
                                    row![
                                        "Add Link:",
                                        pick_list(
                                            (1..=lock.size())
                                                .into_iter()
                                                .filter(|new_link| !slice
                                                    .linked
                                                    .contains_key(new_link)
                                                    && *new_link != index + 1)
                                                .collect::<Vec<_>>(),
                                            None::<usize>,
                                            move |new_link| Message::SetLink(
                                                index,
                                                new_link,
                                                Direction::Same
                                            )
                                        )
                                    ]
                                    .spacing(10),
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
                )
            } else {
                None
            }
        ]
        .width(500)
        .padding(20)
        .spacing(20)
        .into()
    }
}
