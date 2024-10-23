use once_cell::sync::Lazy;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::Frame;
use regex::Regex;
use std::collections::VecDeque;

use crate::app::*;
use lazy_static::lazy_static;

const ROOM_LIST_HEIGHT: usize = 4;

enum MsgType {
    UserMsg,
    OtherMsg,
    SystemMsg,
}

lazy_static! {
    static ref NORMAL_MODE_INSTRUCTION: Vec<&'static str> = {
        let mut str_vec = vec![];
        str_vec.push("'i' => switch to input mode");
        str_vec.push("'q' => exit the room");
        str_vec
    };
    static ref INPUT_MODE_INSTRUCTION: Vec<&'static str> = {
        let mut str_vec = vec![];
        str_vec.push("'Esc' => exit input mode");
        str_vec
    };
}

pub fn ui(
    frame: &mut Frame,
    app: &App,
    chat_room_record: VecDeque<String>,
    chat_room_member: Vec<String>,
) {
    match app.current_screen {
        CurrentScreen::Entry => {
            let instruction_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(3)])
                .split(frame.area());
            let instruction_block = Block::default()
                .title("Instructions")
                .borders(Borders::ALL)
                .style(Style::default());
            let instruction = Paragraph::new("Tab = switch selection, Enter = select, q = quit")
                .block(instruction_block);
            frame.render_widget(instruction, instruction_area[1]);

            let popup_area = centered_rect_with_constant_size(26, 6, instruction_area[0]);
            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(popup_area);

            let mut create_block = Block::default()
                .padding(Padding::horizontal(5))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray));
            let mut join_block = Block::default()
                .padding(Padding::horizontal(6))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray));

            let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

            match app.current_selection {
                CurrentSelection::Create => create_block = create_block.style(active_style),
                CurrentSelection::Join => join_block = join_block.style(active_style),
            }

            let create_text = Text::styled("Create a room", Style::default().fg(Color::Black));
            let join_text = Text::styled("Join a room", Style::default().fg(Color::Black));
            let create_option = Paragraph::new(create_text).block(create_block);
            frame.render_widget(create_option, popup_chunks[0]);

            let join_option = Paragraph::new(join_text).block(join_block);
            frame.render_widget(join_option, popup_chunks[1]);
        }
        CurrentScreen::Create => {
            let area_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            match &app.create_room_error {
                Some(error) => {
                    let error_msg = match error {
                        CreateRoomError::InvalidUsernameChar => {
                            "Username should not contain special characters and whitespace"
                                .to_string()
                        }
                        CreateRoomError::InvalidRoomNameChar => {
                            "Room name can only contain numbers, letters, whitespace, and (!, ?, -, >, <, *)".to_string()
                        }
                        CreateRoomError::ServerError => "Server Error".to_string(),
                        CreateRoomError::InvalidUsernameLength => {
                            "Username's length should be between 1 and 50".to_string()
                        }
                        CreateRoomError::InvalidRoomNameLength => {
                            "Length of room name should be between 1 and 100".to_string()
                        }
                        _ => {
                            "".to_string()
                        }
                    };
                    let error_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));

                    let error = Paragraph::new(error_msg).block(error_block);
                    frame.render_widget(error, area_chunks[0])
                }
                None => {}
            }

            let instruction_block = Block::default()
                .title("Instructions")
                .borders(Borders::ALL)
                .style(Style::default());
            let instruction =
                Paragraph::new("Tab = switch input, Enter = confirm username and room name, Esc = back to main menu")
                    .block(instruction_block);

            frame.render_widget(instruction, area_chunks[2]);

            let popup_area = centered_rect_with_constant_size(40, 6, area_chunks[1]);
            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Fill(1)])
                .split(popup_area);

            let mut username_block = Block::default()
                .title("Please enter a uername")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Black));

            let mut room_name_block = Block::default()
                .title("Please enter a room name")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Black));

            let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

            match app.create_room_input {
                CreateRoomInput::Username => {
                    username_block = username_block.style(active_style);
                }
                CreateRoomInput::RoomName => {
                    room_name_block = room_name_block.style(active_style);
                }
            }

            let username = Paragraph::new(app.username.clone()).block(username_block);
            let room_name = Paragraph::new(app.room_name.clone()).block(room_name_block);

            frame.render_widget(username, popup_chunks[0]);
            frame.render_widget(room_name, popup_chunks[1]);

            if app.password_prompt {
                let prompt_layout = centered_rect_with_constant_size(80, 5, frame.area());
                frame.render_widget(Clear, prompt_layout);
                let pupup_dialog_block = Block::default()
                    .padding(Padding::vertical(1))
                    .title("Press 'y' to set a password, or press 'n' to create a room without password")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightYellow));
                let popup_dialog = Paragraph::new("Do you want to set a password for this room?")
                    .alignment(Alignment::Center)
                    .block(pupup_dialog_block);
                frame.render_widget(popup_dialog, prompt_layout);
            }
        }
        CurrentScreen::CreatePassword => {
            let area_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());
            let instructions_block = Block::default().borders(Borders::ALL).title("Instructions");
            let instructions = Paragraph::new("Enter = confirm, Esc = back to last page")
                .block(instructions_block);
            frame.render_widget(instructions, area_chunks[2]);

            let password_area = centered_rect_with_constant_size(40, 3, area_chunks[1]);
            let password_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightYellow).fg(Color::Black))
                .title("Please enter a password for this room");
            let password = Paragraph::new(app.password.clone()).block(password_block);

            frame.render_widget(password, password_area);

            match &app.create_room_error {
                Some(error) => {
                    let err_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));
                    match error {
                        CreateRoomError::ServerError => {
                            let err = Paragraph::new("Server Error").block(err_block);
                            frame.render_widget(err, area_chunks[0]);
                        }
                        CreateRoomError::InvalidPasswordChar => {
                            let err = Paragraph::new("Length of password should be between 4 and 20 (whitespace not allowed)").block(err_block);
                            frame.render_widget(err, area_chunks[0]);
                        }
                        _ => {}
                    }
                }
                None => {}
            }
        }
        CurrentScreen::Join => {
            let area_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());
            let instruction_block = Block::default().title("Instructions").borders(Borders::ALL);
            let instruction = Paragraph::new(
                "Tab = switch input, Enter = confirm your input, Esc = back to main menu",
            )
            .block(instruction_block);

            frame.render_widget(instruction, area_chunks[2]);

            match &app.join_room_error {
                Some(error) => {
                    let error_msg = match error {
                        JoinRoomError::InvalidUsername => {
                            "Username should not contain special characters or whitespace"
                                .to_string()
                        }
                        JoinRoomError::RoomIdLengthError => "Invalid Room Id format".to_string(),
                        JoinRoomError::RoomNotFound => "Room not found".to_string(),
                        JoinRoomError::InvalidUsernameLength => {
                            "Username's length should be between 1 and 50".to_string()
                        }
                        JoinRoomError::GetRoomListFailed => "Failed to get room list".to_string(),
                        JoinRoomError::WrongPassword => "Wrong password".to_string(),
                        JoinRoomError::ZeroRooms => "There is no room to join".to_string(),
                    };

                    let error_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));

                    let error = Paragraph::new(error_msg).block(error_block);

                    frame.render_widget(error, area_chunks[0]);
                }
                None => {}
            }

            let popup_area = centered_rect_with_constant_size(40, 3, area_chunks[1]);
            //let popup_chunks = Layout::default()
            //    .direction(Direction::Vertical)
            //    .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            //    .split(popup_area);

            //let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

            let username_input_block = Block::default()
                .title("Please enter a uername")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightYellow).fg(Color::Black));

            //let mut room_id_input_block = Block::default()
            //    .title("Please enter the room ID")
            //    .borders(Borders::ALL)
            //    .style(Style::default().bg(Color::DarkGray).fg(Color::Black));

            //match app.join_room_input {
            //    JoinRoomInput::Username => {
            //        username_input_block = username_input_block.style(active_style)
            //    }
            //    JoinRoomInput::RoomId => {
            //        room_id_input_block = room_id_input_block.style(active_style)
            //    }
            //};

            let username = Paragraph::new(app.username.clone()).block(username_input_block);
            //let room_id = Paragraph::new(app.room_id.clone()).block(room_id_input_block);

            frame.render_widget(username, popup_area);
            //frame.render_widget(room_id, popup_chunks[1])
        }
        CurrentScreen::RoomSelect => {
            let area_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let instruction_block = Block::default().title("Instructions").borders(Borders::ALL);
            let instructions = Paragraph::new("Enter = select, 'r' = reload, ArrowUp = prev room, ArrowDown = next room, Esc = back to set username").block(instruction_block);
            frame.render_widget(instructions, area_chunks[3]);

            let height = area_chunks[1].height as usize;
            let mut room_per_page = height / ROOM_LIST_HEIGHT;
            if room_per_page == 0 {
                room_per_page = 1;
            }

            let start_idx = (app.room_idx / room_per_page) * room_per_page;
            let end_idx = (start_idx + room_per_page).min(app.room_lst.len());

            let constraints = vec![
                vec![Constraint::Fill(1)],
                vec![Constraint::Length(ROOM_LIST_HEIGHT as u16); end_idx - start_idx],
                vec![Constraint::Fill(1)],
            ]
            .concat();

            let room_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area_chunks[1]);

            for idx in start_idx..end_idx {
                let cur_room = app.room_lst[idx].clone();
                let title = format!("{}{}", if cur_room.2 { "🔑" } else { "" }, cur_room.0);
                let mut room_block = Block::default().borders(Borders::ALL).title(title);
                if idx == app.room_idx {
                    room_block = room_block.border_style(Style::default().fg(Color::LightYellow));
                }
                let room = Paragraph::new(cur_room.1)
                    .block(room_block)
                    .alignment(Alignment::Center);

                frame.render_widget(room, room_layout[idx - start_idx + 1]);
            }

            let cur_page = (app.room_idx / room_per_page) + 1;
            let mut total_page = (app.room_lst.len() / room_per_page);
            if app.room_lst.len() % room_per_page != 0 {
                total_page += 1;
            }

            let page_display =
                Paragraph::new(format!("{}/{}", cur_page, total_page)).alignment(Alignment::Center);
            frame.render_widget(page_display, area_chunks[2]);

            match &app.join_room_error {
                Some(error) => {
                    let err_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));
                    match error {
                        JoinRoomError::RoomNotFound => {
                            let err = Paragraph::new(
                                "This room doesn't exist anymore, please reload this page with 'r'",
                            )
                            .block(err_block);

                            frame.render_widget(err, area_chunks[0]);
                        }
                        _ => {}
                    }
                }
                None => {}
            }

            //let tmp = Paragraph::new(format!("{}", app.room_idx));
            //frame.render_widget(tmp, frame.area());
        }
        CurrentScreen::PasswordCheck => {
            let area_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let instructions_block = Block::default().borders(Borders::ALL).title("Instructions");
            let instructions = Paragraph::new("Enter = confirm, Esc = back to last page")
                .block(instructions_block);
            frame.render_widget(instructions, area_chunks[2]);

            let password_area = centered_rect_with_constant_size(40, 3, area_chunks[1]);
            let password_block = Block::default()
                .title("Please enter the password of this room")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightYellow).fg(Color::Black));
            let password = Paragraph::new(app.check_passwork.clone()).block(password_block);

            frame.render_widget(password, password_area);

            match &app.join_room_error {
                Some(error) => {
                    let err_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));

                    match error {
                        JoinRoomError::RoomNotFound => {
                            let err = Paragraph::new("This room doesn't exist anymore, please go back to last page and reload").block(err_block);
                            frame.render_widget(err, area_chunks[0]);
                        }
                        JoinRoomError::WrongPassword => {
                            let err = Paragraph::new("Wrong password").block(err_block);
                            frame.render_widget(err, area_chunks[0]);
                        }
                        _ => {}
                    }
                }
                None => {}
            }
        }
        CurrentScreen::Chat => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(80), Constraint::Fill(1)])
                .split(frame.area());

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(2), Constraint::Fill(1)])
                .split(chunks[1]);

            let member_block = Block::default()
                .title("Room members")
                .borders(Borders::ALL)
                .style(Style::default());
            let mut members = Vec::<ListItem>::new();

            for member in chat_room_member.into_iter() {
                members.push(ListItem::new(Line::from(Span::styled(
                    format!("{: <25}", member),
                    Style::default().fg(Color::Green),
                ))));
            }

            let list = List::new(members).block(member_block);
            frame.render_widget(list, right_chunks[0]);

            let instruction_block = Block::default().borders(Borders::ALL).title("Instructions");
            let ins_inner_area = instruction_block.inner(right_chunks[1]);
            let (ins_width, ins_height) = (ins_inner_area.width, ins_inner_area.height);
            match app.chat_room_mode {
                // Todo
                ChatRoomMode::Normal => {
                    let instructions = fit_instructions_into_block(
                        &NORMAL_MODE_INSTRUCTION,
                        ins_width,
                        ins_height,
                    );
                    frame.render_widget(
                        List::new(instructions).block(instruction_block),
                        right_chunks[1],
                    );
                }
                ChatRoomMode::Input => {
                    let instructions =
                        fit_instructions_into_block(&INPUT_MODE_INSTRUCTION, ins_width, ins_height);
                    frame.render_widget(
                        List::new(instructions).block(instruction_block),
                        right_chunks[1],
                    );
                }
            }

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100), Constraint::Min(4)])
                .split(chunks[0]);

            let chat_block = Block::default()
                .title(format!(
                    "Room ID: {}, Room Name: {}",
                    app.room_id.clone(),
                    app.room_name.clone()
                ))
                .borders(Borders::ALL)
                .style(Style::default());

            let chat_inner_area = chat_block.inner(left_chunks[0]);
            let (width, height) = (chat_inner_area.width, chat_inner_area.height);
            let messages = fit_msg_into_chat_block(
                chat_room_record,
                width as usize,
                height as usize,
                &app.username,
            );

            let message_list = List::new(messages).block(chat_block);
            frame.render_widget(message_list, left_chunks[0]);

            let input_block = Block::default()
                .title("Input")
                .borders(Borders::ALL)
                .style(Style::default())
                .border_style(match app.chat_room_mode {
                    ChatRoomMode::Normal => Style::default(),
                    ChatRoomMode::Input => Style::default().fg(Color::Blue),
                });
            let inner_area = input_block.inner(left_chunks[1]);
            let input_width = inner_area.width as usize;

            let input_clone = app.input.clone();
            if input_width < input_clone.len() {
                let input_text = Paragraph::new(
                    &input_clone[input_clone.len() - input_width..input_clone.len()],
                )
                .block(input_block);
                frame.render_widget(input_text, left_chunks[1]);
            } else {
                let input_text = Paragraph::new(input_clone).block(input_block);
                frame.render_widget(input_text, left_chunks[1]);
            }

            if app.exiting {
                let popup_dialog_area = centered_rect_with_constant_size(80, 5, frame.area());
                frame.render_widget(Clear, popup_dialog_area);
                let pupup_dialog_block = Block::default()
                    .padding(Padding::vertical(1))
                    .title("Press 'y' to exit the room, or press 'n' to cancel")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red));
                let popup_dialog = Paragraph::new("Are you sure you want to leave the chat room?")
                    .alignment(Alignment::Center)
                    .block(pupup_dialog_block);
                frame.render_widget(popup_dialog, popup_dialog_area);
            }
        }
        CurrentScreen::Exiting => {}
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn centered_rect_with_constant_size(size_x: u16, size_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(size_y),
            Constraint::Fill(1),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(size_x),
            Constraint::Fill(1),
        ])
        .split(popup_layout[1])[1]
}

fn fit_newest_msg_into_screen(
    mut msg_vec: VecDeque<String>,
    width: usize,
    mut height: usize,
) -> VecDeque<String> {
    let mut res = VecDeque::new();
    while let Some(msg) = msg_vec.pop_back() {
        let lines = (msg.len() / width) + if msg.len() % width > 0 { 1 } else { 0 };
        if lines > height {
            break;
        } else {
            res.push_front(msg);
            height -= lines;
        }
    }
    res
}

fn fit_msg_into_chat_block<'a>(
    mut msg_vec: VecDeque<String>,
    width: usize,
    height: usize,
    username: &str,
) -> VecDeque<ListItem<'a>> {
    let mut res: VecDeque<ListItem> = VecDeque::new();
    while res.len() < height && !msg_vec.is_empty() {
        let msg = msg_vec.pop_back().unwrap();
        let user_msg = match extract_username(&msg) {
            Some(name) => {
                if name.as_str() == username {
                    MsgType::UserMsg
                } else {
                    MsgType::OtherMsg
                }
            }
            None => MsgType::SystemMsg,
        };

        if msg.len() <= width {
            res.push_front(ListItem::new(Line::from(Span::styled(
                msg,
                match user_msg {
                    MsgType::UserMsg => Style::default().fg(Color::LightYellow),
                    MsgType::OtherMsg => Style::default().fg(Color::LightGreen),
                    MsgType::SystemMsg => Style::default(),
                },
            ))));
        } else {
            let split_msg = msg.split(" ").collect::<Vec<&str>>();

            let mut tmp_str = String::new();
            let mut tmp_vec = vec![];

            for part in split_msg.into_iter() {
                if tmp_str.len() + part.len() > width {
                    tmp_vec.push(tmp_str.clone());
                    tmp_str.clear();
                }
                tmp_str += part;
                tmp_str += " ";
            }

            if !tmp_str.is_empty() {
                tmp_vec.push(tmp_str);
            }

            if res.len() + tmp_vec.len() > height {
                break;
            } else {
                while !tmp_vec.is_empty() {
                    res.push_front(ListItem::new(Line::from(Span::styled(
                        tmp_vec.pop().unwrap(),
                        match user_msg {
                            MsgType::UserMsg => Style::default().fg(Color::LightYellow),
                            MsgType::OtherMsg => Style::default().fg(Color::LightGreen),
                            MsgType::SystemMsg => Style::default(),
                        },
                    ))));
                }
            }
        }
    }
    res
}

fn fit_instructions_into_block<'a>(
    instructions: &Vec<&str>,
    width: u16,
    height: u16,
) -> Vec<ListItem<'a>> {
    let (width, height) = (width as usize, height as usize);

    let mut res = vec![];

    for instruction in instructions.iter() {
        let mut tmp_str = String::new();
        let split_parts = instruction.split(" ").collect::<Vec<&str>>();

        for word in split_parts.into_iter() {
            if tmp_str.len() + word.len() <= width {
                tmp_str += word;
                tmp_str += " ";
            } else {
                res.push(ListItem::new(Line::from(Span::styled(
                    tmp_str.clone(),
                    Style::default(),
                ))));

                tmp_str.clear();
                tmp_str += word;
                tmp_str += " ";
            }
        }

        if !tmp_str.is_empty() {
            res.push(ListItem::new(Line::from(Span::styled(
                tmp_str.clone(),
                Style::default(),
            ))));
        }
        res.push(ListItem::new(Line::from(" ")));
    }

    res
}

fn extract_username(hay: &str) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"^([^!@#$%\^\&\*\(\)\+=\[\]\{\}:;'"/<>|\\`~\?,\.\s]+):"#).unwrap()
    });
    if let Some(cap) = RE.captures(hay) {
        if let Some(first) = cap.get(1) {
            return Some(first.as_str().to_string());
        }
    }
    None
}
