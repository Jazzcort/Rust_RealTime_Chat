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
use once_cell::sync::Lazy;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{
    self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use regex::Regex;
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

    // dbg!(app);

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    remote_server: &str,
) -> std::io::Result<()> {
    loop {
        let chat_room_record_arc = app.chat_room_record.clone();
        let chat_member_arc = app.chat_room_member.clone();

        let chat_room_record_handle = chat_room_record_arc.lock().await;
        let chat_room_record = chat_room_record_handle.clone();
        drop(chat_room_record_handle);
        // let chat_room_record = chat_room_record.into_iter().rev().collect::<Vec<String>>();

        let chat_room_member_handle = chat_member_arc.lock().await;
        let chat_members = chat_room_member_handle.clone();
        drop(chat_room_member_handle);

        let abandon_arc = app.abandon.clone();
        let abandon_handle = abandon_arc.lock().await;
        if *abandon_handle {
            // return Ok(());
            app.reinitialize();
        }
        drop(abandon_handle);

        terminal.draw(|f| ui(f, app, chat_room_record, chat_members))?;

        if !poll(std::time::Duration::from_millis(350))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            // if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
            //     return Ok(());
            // }

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
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                CurrentScreen::Create => {
                    if app.password_prompt {
                        match key.code {
                            KeyCode::Esc => {
                                app.password_prompt = false;
                                app.create_room_error = None;
                            }
                            KeyCode::Char('y') => {
                                app.current_screen = CurrentScreen::CreatePassword;
                                app.create_room_error = None;
                            }
                            KeyCode::Char('n') => {
                                if let Ok((msg_pipe, room_id)) = Client::create_room(
                                    app.username.clone(),
                                    app.room_name.clone(),
                                    None,
                                    app.chat_room_record.clone(),
                                    app.chat_room_member.clone(),
                                    app.abandon.clone(),
                                    app.record_size,
                                    remote_server,
                                )
                                .await
                                {
                                    app.msg_pipe = Some(msg_pipe);
                                    app.enter_room(room_id);
                                } else {
                                    app.create_room_error = Some(CreateRoomError::ServerError);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Backspace => match app.create_room_input {
                                CreateRoomInput::Username => {
                                    app.username.pop();
                                    app.create_room_error = None;
                                }
                                CreateRoomInput::RoomName => {
                                    app.room_name.pop();
                                    app.create_room_error = None
                                }
                            },
                            KeyCode::Tab => match app.create_room_input {
                                CreateRoomInput::Username => {
                                    app.create_room_input = CreateRoomInput::RoomName
                                }
                                CreateRoomInput::RoomName => {
                                    app.create_room_input = CreateRoomInput::Username
                                }
                            },
                            KeyCode::Enter => {
                                if app.username.len() < 1 || app.username.len() > 50 {
                                    app.create_room_error =
                                        Some(CreateRoomError::InvalidUsernameLength);
                                    continue;
                                }
                                if app.room_name.len() < 1 || app.room_name.len() > 100 {
                                    app.create_room_error =
                                        Some(CreateRoomError::InvalidRoomNameLength);
                                    continue;
                                }

                                if !is_valid_string(&app.username) {
                                    app.create_room_error =
                                        Some(CreateRoomError::InvalidUsernameChar);
                                    continue;
                                }
                                if !is_valid_string_with_whitespace(&app.room_name) {
                                    app.create_room_error =
                                        Some(CreateRoomError::InvalidRoomNameChar);
                                    continue;
                                }
                                app.password_prompt = true;
                            }
                            KeyCode::Char(value) => match app.create_room_input {
                                CreateRoomInput::Username => {
                                    app.username.push(value);
                                    app.create_room_error = None;
                                }
                                CreateRoomInput::RoomName => {
                                    app.room_name.push(value);
                                    app.create_room_error = None;
                                }
                            },
                            KeyCode::Esc => {
                                app.username.clear();
                                app.room_name.clear();
                                app.create_room_error = None;
                                app.current_screen = CurrentScreen::Entry;
                                app.create_room_input = CreateRoomInput::Username;
                            }
                            _ => {}
                        }
                    }
                }
                CurrentScreen::CreatePassword => match key.code {
                    KeyCode::Char(value) => {
                        app.password.push(value);
                        app.create_room_error = None;
                    }
                    KeyCode::Backspace => {
                        app.password.pop();
                        app.create_room_error = None;
                    }
                    KeyCode::Esc => {
                        app.password.clear();
                        app.current_screen = CurrentScreen::Create;
                        app.create_room_error = None;
                        app.password_prompt = false;
                    }
                    KeyCode::Enter => {
                        if app.password.len() < 4 || app.password.len() > 20 {
                            app.create_room_error = Some(CreateRoomError::InvalidPasswordChar);
                            continue;
                        }

                        if has_whitespace(&app.password) {
                            app.create_room_error = Some(CreateRoomError::InvalidPasswordChar);
                            continue;
                        }

                        if let Ok((msg_pipe, room_id)) = Client::create_room(
                            app.username.clone(),
                            app.room_name.clone(),
                            Some(app.password.clone()),
                            app.chat_room_record.clone(),
                            app.chat_room_member.clone(),
                            app.abandon.clone(),
                            app.record_size,
                            remote_server,
                        )
                        .await
                        {
                            app.msg_pipe = Some(msg_pipe);
                            app.enter_room(room_id);
                        } else {
                            app.create_room_error = Some(CreateRoomError::ServerError);
                        }
                    }
                    _ => {}
                },
                CurrentScreen::Join => match key.code {
                    KeyCode::Backspace => {
                        app.join_room_error = None;
                        match app.join_room_input {
                            JoinRoomInput::Username => {
                                app.username.pop();
                            }
                            JoinRoomInput::RoomId => {
                                app.room_id.pop();
                            }
                        }
                    }
                    KeyCode::Char(value) => {
                        app.join_room_error = None;
                        match app.join_room_input {
                            JoinRoomInput::Username => {
                                app.username.push(value);
                            }
                            JoinRoomInput::RoomId => {
                                app.room_id.push(value);
                            }
                        }
                    }
                    KeyCode::Tab => match app.join_room_input {
                        JoinRoomInput::Username => {
                            app.join_room_input = JoinRoomInput::RoomId;
                        }
                        JoinRoomInput::RoomId => {
                            app.join_room_input = JoinRoomInput::Username;
                        }
                    },
                    KeyCode::Enter => {
                        if app.room_id.len() != 8 {
                            app.join_room_error = Some(JoinRoomError::RoomIdLengthError);
                            continue;
                        }

                        if app.username.len() < 1 || app.username.len() > 50 {
                            app.join_room_error = Some(JoinRoomError::InvalidUsernameLength);
                            continue;
                        }

                        if !is_valid_string(&app.username) {
                            app.join_room_error = Some(JoinRoomError::InvalidUsername);
                            continue;
                        }

                        if let Ok(room_lst) = Client::get_room_list(remote_server).await {
                            app.room_lst = room_lst;
                        } else {
                            app.join_room_error = Some(JoinRoomError::GetRoomListFailed);
                        }

                        if app.room_lst.len() != 0 {
                            app.current_screen = CurrentScreen::RoomSelect;
                        } else {
                            app.join_room_error = Some(JoinRoomError::ZeroRooms);
                        }
                    }
                    KeyCode::Esc => {
                        app.username.clear();
                        app.room_id.clear();
                        app.join_room_error = None;
                        app.current_screen = CurrentScreen::Entry;
                    }
                    _ => {}
                },
                CurrentScreen::RoomSelect => match key.code {
                    KeyCode::Enter => {
                        let select_room = app.room_lst[app.room_idx].clone();

                        if select_room.2 {
                            app.current_screen = CurrentScreen::PasswordCheck;
                        } else {
                            if let Ok((msg_pipe, room_id)) = Client::enter_room(
                                app.username.clone(),
                                select_room.0,
                                None,
                                app.chat_room_record.clone(),
                                app.chat_room_member.clone(),
                                app.abandon.clone(),
                                app.record_size,
                                remote_server,
                            )
                            .await
                            {
                                app.msg_pipe = Some(msg_pipe);
                                app.enter_room(room_id);
                            } else {
                                app.join_room_error = Some(JoinRoomError::RoomNotFound)
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Join;
                    }
                    KeyCode::Char('r') => {
                        if let Ok(room_lst) = Client::get_room_list(remote_server).await {
                            if room_lst.len() != 0 {
                                app.room_lst = room_lst;
                                app.room_idx = 0;
                            } else {
                                app.room_lst.clear();
                                app.room_idx = 0;
                                app.join_room_error = Some(JoinRoomError::ZeroRooms);
                                app.current_screen = CurrentScreen::Join;
                            }
                        } else {
                            app.room_lst.clear();
                            app.room_idx = 0;
                            app.join_room_error = Some(JoinRoomError::GetRoomListFailed);
                            app.current_screen = CurrentScreen::Join;
                        }
                    }
                    KeyCode::Up => match app.room_idx.checked_sub(1) {
                        Some(val) => app.room_idx = val,
                        None => app.room_idx = 0,
                    },
                    KeyCode::Down => {
                        if app.room_lst.len() > app.room_idx + 1 {
                            app.room_idx += 1;
                        }
                    }
                    _ => {}
                },
                CurrentScreen::PasswordCheck => match key.code {
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::RoomSelect;
                        app.check_passwork.clear();
                        app.join_room_error = None;
                    }
                    KeyCode::Char(value) => {
                        app.check_passwork.push(value);
                        app.join_room_error = None;
                    }
                    KeyCode::Backspace => {
                        app.check_passwork.pop();
                        app.join_room_error = None;
                    }
                    KeyCode::Enter => {
                        match Client::enter_room(
                            app.username.clone(),
                            app.room_lst[app.room_idx].0.clone(),
                            Some(app.check_passwork.clone()),
                            app.chat_room_record.clone(),
                            app.chat_room_member.clone(),
                            app.abandon.clone(),
                            app.record_size,
                            remote_server,
                        )
                        .await
                        {
                            Ok((msg_pipe, room_id)) => {
                                app.msg_pipe = Some(msg_pipe);
                                app.enter_room(room_id);
                            }
                            Err(e) => match e.kind() {
                                std::io::ErrorKind::BrokenPipe => {
                                    app.join_room_error = Some(JoinRoomError::RoomNotFound);
                                }
                                std::io::ErrorKind::InvalidInput => {
                                    app.join_room_error = Some(JoinRoomError::WrongPassword);
                                }
                                _ => {}
                            },
                        }
                    }
                    _ => {}
                },
                CurrentScreen::Chat => {
                    if app.exiting {
                        match key.code {
                            KeyCode::Char('n') => app.exiting = false,
                            KeyCode::Char('y') => {
                                app.msg_pipe = None;
                                // app.room_id = String::new();
                                // app.current_screen = CurrentScreen::Entry;
                            }
                            _ => {}
                        }
                    } else {
                        match app.chat_room_mode {
                            ChatRoomMode::Normal => match key.code {
                                KeyCode::Char('i') => {
                                    app.chat_room_mode = ChatRoomMode::Input;
                                }
                                KeyCode::Char('q') => {
                                    app.exiting = true;
                                }
                                _ => {}
                            },
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
                        }
                    }
                }
                CurrentScreen::Exiting => {}
            }
        }
    }
    Ok(())
}

fn is_valid_string(s: &str) -> bool {
    static USERNAME_RESTRICT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"([!@#$%\^\&\*\(\)\+=\[\]\{\}:;'"/<>|\\`~\?,\.\s]+)"#).unwrap());

    !USERNAME_RESTRICT.is_match(s)
}

fn is_valid_string_with_whitespace(s: &str) -> bool {
    static USERNAME_RESTRICT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"([@#$%\^\&\(\)\+=\[\]\{\}:;'"/|\\`~,\.]+)"#).unwrap());

    !USERNAME_RESTRICT.is_match(s)
}

fn has_whitespace(s: &str) -> bool {
    static USERNAME_RESTRICT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"([\s]+)"#).unwrap());

    USERNAME_RESTRICT.is_match(s)
}
