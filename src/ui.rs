use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::Frame;
use style::Styled;
use std::collections::VecDeque;

use crate::app::*;

pub fn ui(
    frame: &mut Frame,
    app: &App,
    chat_room_record: Vec<String>,
    chat_room_member: Vec<String>,
) {
    match app.current_screen {
        CurrentScreen::Entry => {
            let popup_area = centered_rect_with_constant_size(26, 6, frame.area());
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
            let popup_area = centered_rect_with_constant_size(40, 3, frame.area());
            let input_block = Block::default()
                .title("Please enter a uername")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightYellow).fg(Color::Black));
            let username = Paragraph::new(app.username.clone()).block(input_block);
            frame.render_widget(username, popup_area);
        }
        CurrentScreen::Join => {
            let popup_area = centered_rect_with_constant_size(40, 6, frame.area());
            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Fill(1)])
                .split(popup_area);

            let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

            let mut username_input_block = Block::default()
                .title("Please enter a uername")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Black));

            let mut room_id_input_block = Block::default()
                .title("Please enter the room ID")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Black));

            match app.join_room_input {
                JoinRoomInput::Username => {
                    username_input_block = username_input_block.style(active_style)
                }
                JoinRoomInput::RoomId => {
                    room_id_input_block = room_id_input_block.style(active_style)
                }
            };

            let username = Paragraph::new(app.username.clone()).block(username_input_block);
            let room_id = Paragraph::new(app.room_id.clone()).block(room_id_input_block);

            frame.render_widget(username, popup_chunks[0]);
            frame.render_widget(room_id, popup_chunks[1])
        }
        CurrentScreen::Chat => {
            let width = frame.area().width;
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(85), Constraint::Fill(1)])
                .split(frame.area());

            let member_block = Block::default()
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
            frame.render_widget(list, chunks[1]);

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100), Constraint::Min(4)])
                .split(chunks[0]);

            let chat_block = Block::default()
                .title(app.room_id.clone())
                .borders(Borders::ALL)
                .style(Style::default());

            let mut messages = Vec::<ListItem>::new();
            for message in chat_room_record.into_iter() {
                if message.len() as u16 > width {
                    let msg_chunks = message
                        .chars()
                        .collect::<Vec<char>>()
                        .chunks(width as usize)
                        .map(|x| x.iter().collect::<String>())
                        .collect::<Vec<String>>();
                    for msg_chunk in msg_chunks.into_iter() {
                        messages.push(ListItem::new(Line::from(Span::styled(
                            msg_chunk,
                            Style::default().fg(Color::Yellow),
                        ))))
                    }
                } else {
                    messages.push(ListItem::new(Line::from(Span::styled(
                        message,
                        Style::default().fg(Color::Yellow),
                    ))));
                }
            }
            let message_list = List::new(messages).block(chat_block);
            frame.render_widget(message_list, left_chunks[0]);

            let input_block = Block::default().title("Input").borders(Borders::ALL).style(Style::default()).border_style(match app.chat_room_mode {
                ChatRoomMode::Normal => {
                    Style::default()
                }
                ChatRoomMode::Input => {
                    Style::default().fg(Color::Blue)
                }
            });
            let input_text = Paragraph::new(app.input.clone()).block(input_block);
            frame.render_widget(input_text, left_chunks[1]);
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
