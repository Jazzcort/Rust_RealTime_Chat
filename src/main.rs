mod app;
mod client;
mod command_parser;
mod ui;
mod util;

use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{stdin, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use std::net::TcpStream;
use crate::app::*;
use crate::client::Client;
use crate::command_parser::{Args, Operation};
use crate::ui::*;
use crate::util::read_buf;
use clap::Parser;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{
    self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> Result<(), Error> {

    let remote_server = dotenv!("REMOTE_SERVER");

    // let args = Args::parse();

    // #[allow(unused_assignments)]
    // let mut client: Option<Sender<String>> = None;

    // match &args.operation {
    //     Operation::Create { username } => {
    //         client = Some(Client::create_room(username.clone()).await?);
    //     }
    //     Operation::Join { username, room_id } => {
    //         client = Some(Client::enter_room(username.clone(), room_id.clone()).await?);
    //     }
    // }

    // if let Some(client) = client {
    //     loop {
    //         let mut user_input = String::new();
    //         let _ = stdin().read_line(&mut user_input)?;

    //         let _ = client.send(user_input.clone()).await;
    //     }
    // } else {
    //     println!("Something went wrong");
    //     Ok(())
    // }



    enable_raw_mode()?;
    let mut stderr = std::io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, remote_server).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, remote_server: &str) -> std::io::Result<()> {
    let chat_room_record_arc = app.chat_room_record.clone();
    loop {
        let chat_room_record_handle = chat_room_record_arc.lock().await;
        let chat_room_record = chat_room_record_handle.clone();
        drop(chat_room_record_handle);
        let chat_room_record = chat_room_record.into_iter().rev().collect::<Vec<String>>();
        let chat_members = vec![
            "jazzcort".to_string(),
            "erin".to_string(),
            "Taco".to_string(),
        ];
        let abandon_arc = app.abandon.clone();
        let abandon_handle = abandon_arc.lock().await;
        if *abandon_handle {
            return Ok(());
            // app.current_screen = CurrentScreen::Exiting;
        }

        terminal.draw(|f| ui(f, app, chat_room_record, chat_members))?;

        if !poll(std::time::Duration::from_millis(350))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }

            match app.current_screen {
                CurrentScreen::Entry => match key.code {
                    KeyCode::Tab => match app.current_selection {
                        CurrentSelection::Create => app.current_selection = CurrentSelection::Join,
                        CurrentSelection::Join => app.current_selection = CurrentSelection::Create,
                    },
                    KeyCode::Enter => match app.current_selection {
                        CurrentSelection::Create => app.current_screen = CurrentScreen::Create,
                        CurrentSelection::Join => app.current_screen = CurrentScreen::Join,
                    },
                    _ => {}
                },
                CurrentScreen::Create => match key.code {
                    KeyCode::Backspace => {
                        app.username.pop();
                    }
                    KeyCode::Enter => {
                        if let Ok((msg_pipe, mut abandon_pipe, room_id)) = Client::create_room(
                            app.username.clone(),
                            app.chat_room_record.clone(),
                            app.record_size,
                            remote_server
                        )
                        .await
                        {
                            app.msg_pipe = Some(msg_pipe);
                            let abandon_copy = app.abandon.clone();
                            tokio::task::spawn(async move {
                                loop {
                                    if let Some(signal) = abandon_pipe.recv().await {
                                        if signal == 0 {
                                            let mut abandon_handle = abandon_copy.lock().await;
                                            *abandon_handle = true;
                                            break;
                                        }
                                    }
                                }
                            });
                            app.enter_room(room_id);
                        }
                    }
                    KeyCode::Char(value) => {
                        app.username.push(value);
                    }
                    KeyCode::Esc => {
                        app.username.clear();
                        app.current_screen = CurrentScreen::Entry
                    }
                    _ => {}
                },
                CurrentScreen::Join => match key.code {
                    KeyCode::Backspace => match app.join_room_input {
                        JoinRoomInput::Username => {
                            app.username.pop();
                        }
                        JoinRoomInput::RoomId => {
                            app.room_id.pop();
                        }
                    },
                    KeyCode::Char(value) => match app.join_room_input {
                        JoinRoomInput::Username => {
                            app.username.push(value);
                        }
                        JoinRoomInput::RoomId => {
                            app.room_id.push(value);
                        }
                    },
                    KeyCode::Tab => match app.join_room_input {
                        JoinRoomInput::Username => {
                            app.join_room_input = JoinRoomInput::RoomId;
                        }
                        JoinRoomInput::RoomId => {
                            app.join_room_input = JoinRoomInput::Username;
                        }
                    },
                    KeyCode::Enter => {
                        if let Ok((msg_pipe, mut abandon_pipe, room_id)) = Client::enter_room(
                            app.username.clone(),
                            app.room_id.clone(),
                            app.chat_room_record.clone(),
                            app.record_size,
                            remote_server
                        )
                        .await
                        {
                            app.msg_pipe = Some(msg_pipe);
                            let abandon_copy = app.abandon.clone();
                            tokio::task::spawn(async move {
                                loop {
                                    if let Some(signal) = abandon_pipe.recv().await {
                                        if signal == 0 {
                                            let mut abandon_handle = abandon_copy.lock().await;
                                            *abandon_handle = true;
                                            break;
                                        }
                                    }
                                }
                            });
                            app.enter_room(room_id);
                        }
                    }
                    KeyCode::Esc => {
                        app.username.clear();
                        app.room_id.clear();
                        app.current_screen = CurrentScreen::Entry;
                    }
                    _ => {}
                },
                CurrentScreen::Chat => match app.chat_room_mode {
                    ChatRoomMode::Normal => match key.code {
                        KeyCode::Char('i') => {
                            app.chat_room_mode = ChatRoomMode::Input;
                        }
                        _ => {}
                    }
                    ChatRoomMode::Input => match key.code {
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Char(value) => {
                            app.input.push(value);
                        }
                        KeyCode::Enter => {
                            let a = app.msg_pipe.as_ref().unwrap();
                            let _ = a.send(app.input.clone()).await;
                            app.input.clear();
                        }
                        KeyCode::Esc => {
                            app.input.clear();
                            app.chat_room_mode = ChatRoomMode::Normal;
                        }
                        _ => {}
                    },
                },
                CurrentScreen::Exiting => return Ok(()),
            }
        }
    }
}
