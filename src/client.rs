use crate::util::{clear_buf, read_buf};
use std::collections::VecDeque;
use std::io::{stdin, Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream};
use tokio::select;
use tokio::sync::mpsc;
use tokio::task;
pub(crate) struct Client;

impl Client {
    pub(crate) async fn create_room(username: String, chat_room_record: Arc<Mutex<VecDeque<String>>>, record_size: u32, remote_server: &str) -> Result<(mpsc::Sender<String>, mpsc::Receiver<u8>, String), Error> {
        let mut stream = TcpStream::connect(remote_server).await?;
        // let mut stream = TcpStream::connect("127.0.0.1:20130").await?;
        let (tx, rx) = mpsc::channel::<String>(10);
        let (tx_abandon, rx_abandon) = mpsc::channel::<u8>(3);

        let _ = stream
            .write_all(format!("create\r\n{}", username).as_bytes())
            .await?;
        let mut buf = [0_u8; 256];
        let _r_size = stream.read(&mut buf).await?;
        let room_id = read_buf(&buf);
        // dbg!(&room_id);

        Self::start_chat(stream, rx, chat_room_record, tx_abandon, record_size);

        // task::spawn(async move {
        //     let (reader, mut writer) = stream.split();
        //     let mut buf = vec![0_u8; 1024];

        //     let mut reader = BufReader::new(reader);


        //     loop {
        //         tokio::select! {
        //             // Read message from stream
        //             result = reader.read(&mut buf) => {
        //                 let bytes_read = result.unwrap();
        //                 if bytes_read == 0 {
        //                     // println!("Server Disconnected");
        //                     break;
        //                 }

        //                 let msg = read_buf(&buf);
        //                 // Send message to app --> Todo with Arc
        //                 let mut room_record_handle = chat_room_record.lock().await;
        //                 room_record_handle.push_back(msg.trim_end().to_string());
        //                 drop(room_record_handle);;
        //                 clear_buf(&mut buf);
        //             }
        //             // Read user input
        //             result = rx.recv() => {
        //                 if let Some(user_input) = result {
        //                     let _ = writer.write_all(user_input.as_bytes()).await;
        //                 }
        //             }
        //         }
        //     }
        //     // Abandon pipe
        //     let _ = tx_abandon.send(0).await;
        // });

        Ok((tx, rx_abandon, room_id))
    }

    // Todo
    pub(crate) async fn enter_room(username: String, room_id: String, chat_room_record: Arc<Mutex<VecDeque<String>>>, record_size: u32, remote_server: &str) -> Result<(mpsc::Sender<String>, mpsc::Receiver<u8>, String), Error> {
        let mut stream = TcpStream::connect(remote_server).await?;
        // let mut stream = TcpStream::connect("127.0.0.1:20130").await?;
        let (tx, rx) = mpsc::channel::<String>(10);
        let (tx_abandon, rx_abandon) = mpsc::channel::<u8>(3);

        let _ = stream
            .write_all(format!("join\r\n{}\r\n{}", username, room_id).as_bytes())
            .await?;
        let mut buf = [0_u8; 256];
        let _r_size = stream.read(&mut buf).await?;
        let room_id = read_buf(&buf);

        Self::start_chat(stream, rx, chat_room_record, tx_abandon, record_size);

        Ok((tx, rx_abandon, room_id))
    }

    fn start_chat(mut stream: TcpStream, mut rx: mpsc::Receiver<String>, chat_room_record: Arc<Mutex<VecDeque<String>>>, tx_abandon: mpsc::Sender<u8>, record_size: u32) {
        task::spawn(async move {
            let (reader, mut writer) = stream.split();
            let mut buf = vec![0_u8; 1024];
            let record_size = record_size as usize;
            let mut reader = BufReader::new(reader);

            loop {
                tokio::select! {
                    result = reader.read(&mut buf) => {
                        let bytes_read = result.unwrap();
                        if bytes_read == 0 {
                            break;
                        }

                        let msg = read_buf(&buf);
                        let mut room_record_handle = chat_room_record.lock().await;
                        room_record_handle.push_back(msg.trim_end().to_string());
                        if room_record_handle.len() > record_size {
                            room_record_handle.pop_front();
                        }
                        drop(room_record_handle);
                        clear_buf(&mut buf);
                    }
                    result = rx.recv() => {
                        if let Some(user_input) = result {
                            let _ = writer.write_all(user_input.as_bytes()).await;
                        }
                    }
                }
            }
            let _ = tx_abandon.send(0).await;
        });
    }
}
