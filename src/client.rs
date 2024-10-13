use crate::util::{clear_buf, read_buf};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::VecDeque;
use std::io::{stdin, Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task;
pub(crate) struct Client;

impl Client {
    pub(crate) async fn create_room(
        username: String,
        chat_room_record: Arc<Mutex<VecDeque<String>>>,
        chat_room_member: Arc<Mutex<Vec<String>>>,
        abandon_handle: Arc<Mutex<bool>>,
        record_size: u32,
        remote_server: &str,
    ) -> Result<(mpsc::Sender<String>, mpsc::Receiver<u8>, String), Error> {
        let mut stream = TcpStream::connect(remote_server).await?;
        // let mut stream = TcpStream::connect("127.0.0.1:20130").await?;
        let (tx, rx) = mpsc::channel::<String>(10);
        let (tx_abandon, rx_abandon) = mpsc::channel::<u8>(3);

        let _ = stream
            .write_all(format!("create\r\n{}", username).as_bytes())
            .await?;

        let (reader, _) = stream.split();
        let mut reader = BufReader::new(reader);

        let buffer = Vec::from(reader.fill_buf().await.unwrap());
        reader.consume(buffer.len());
        let room_id = read_buf(&buffer);

        Self::start_chat(
            stream,
            rx,
            chat_room_record,
            chat_room_member,
            abandon_handle,
            tx_abandon,
            record_size,
        );

        Ok((tx, rx_abandon, room_id))
    }

    pub(crate) async fn enter_room(
        username: String,
        room_id: String,
        chat_room_record: Arc<Mutex<VecDeque<String>>>,
        chat_room_member: Arc<Mutex<Vec<String>>>,
        abandon_handle: Arc<Mutex<bool>>,
        record_size: u32,
        remote_server: &str,
    ) -> Result<(mpsc::Sender<String>, mpsc::Receiver<u8>, String), Error> {
        let mut stream = TcpStream::connect(remote_server).await?;
        // let mut stream = TcpStream::connect("127.0.0.1:20130").await?;
        let (tx, rx) = mpsc::channel::<String>(10);
        let (tx_abandon, rx_abandon) = mpsc::channel::<u8>(3);

        let _ = stream
            .write_all(format!("join\r\n{}\r\n{}", username, room_id).as_bytes())
            .await?;

        let (reader, _) = stream.split();
        let mut reader = BufReader::new(reader);
        let buffer = Vec::from(reader.fill_buf().await?);
        reader.consume(buffer.len());

        let x = String::from_utf8_lossy(&buffer).to_string();

        if &x == "@#$failed" {
            return Err(Error::new(ErrorKind::BrokenPipe, "Room not found"));
        }

        let room_id_and_people = x.split("\r\n").collect::<Vec<&str>>();
        let room_id = room_id_and_people[0].to_string();

        let mut chat_room_member_handle = chat_room_member.lock().await;
        for i in 1..room_id_and_people.len() {
            chat_room_member_handle.push(room_id_and_people[i].to_string());
        }
        drop(chat_room_member_handle);

        Self::start_chat(
            stream,
            rx,
            chat_room_record,
            chat_room_member,
            abandon_handle,
            tx_abandon,
            record_size,
        );

        Ok((tx, rx_abandon, room_id))
    }

    fn start_chat(
        mut stream: TcpStream,
        mut rx: mpsc::Receiver<String>,
        chat_room_record: Arc<Mutex<VecDeque<String>>>,
        chat_room_member: Arc<Mutex<Vec<String>>>,
        abandon_handle: Arc<Mutex<bool>>,
        tx_abandon: mpsc::Sender<u8>,
        record_size: u32,
    ) {
        task::spawn(async move {
            let (reader, mut writer) = stream.split();
            let record_size = record_size as usize;
            let mut reader = BufReader::new(reader);

            loop {
                tokio::select! {
                    result = reader.fill_buf() => {
                        if result.is_ok() {
                            let buffer = Vec::from(result.unwrap());
                            reader.consume(buffer.len());
                            if buffer.len() == 0 {
                                break;
                            }

                            let msg = String::from_utf8_lossy(&buffer);

                            if let Some(disconnected_user) = match_regex_left(&msg) {
                                let mut chat_room_member_handle = chat_room_member.lock().await;
                                if let Some(pos) = chat_room_member_handle.iter().position(|x| *x == disconnected_user) {
                                    chat_room_member_handle.remove(pos);
                                }
                            } else if let Some(joined_user) = match_regex_join(&msg) {
                                let mut chat_room_member_handle = chat_room_member.lock().await;
                                chat_room_member_handle.push(joined_user);
                            }

                            let mut room_record_handle = chat_room_record.lock().await;
                            room_record_handle.push_back(msg.trim_end().to_string());
                            if room_record_handle.len() > record_size {
                                room_record_handle.pop_front();
                            }
                            drop(room_record_handle);

                        } else {
                            // Stream timeout or reset
                            break;
                        }

                    }
                    result = rx.recv() => {
                        if let Some(user_input) = result {
                            let _ = writer.write_all(user_input.as_bytes()).await;
                        } else {
                            break;
                        }
                    }
                }
            }
            // let _ = tx_abandon.send(0).await;
            let mut abandon = abandon_handle.lock().await;
            *abandon = true;
            // dbg!("client closed!");
        });
    }
}

fn match_regex_left(hay: &str) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"^([^!@#$%\^\&\*\(\)\+=\[\]\{\}:;'"/<>|\\`~\?,\.\s]+) has left the chat room"#)
            .unwrap()
    });
    if let Some(cap) = RE.captures(hay) {
        if let Some(first) = cap.get(1) {
            return Some(first.as_str().to_string());
        }
    }
    None
}

fn match_regex_join(hay: &str) -> Option<String> {
    static RE2: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"^([^!@#$%\^\&\*\(\)\+=\[\]\{\}:;'"/<>|\\`~\?,\.\s]+) has joined the chat room"#,
        )
        .unwrap()
    });
    if let Some(cap) = RE2.captures(hay) {
        if let Some(first) = cap.get(1) {
            return Some(first.as_str().to_string());
        }
    }
    None
}
