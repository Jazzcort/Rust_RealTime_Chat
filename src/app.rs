use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
pub enum CurrentScreen {
    Entry,
    Create,
    Join,
    Chat,
    Exiting
}

pub enum CurrentSelection {
    Create,
    Join
}

pub enum JoinRoomInput {
    Username,
    RoomId
}

pub enum ChatRoomMode {
    Input,
    Normal
}

pub struct App {
    pub input: String,
    pub chat_room_record: Arc<Mutex<VecDeque<String>>>,
    pub chat_room_member: Arc<Mutex<Vec<String>>>,
    pub record_size: u32,
    pub current_screen: CurrentScreen,
    pub editing: bool,
    pub room_id: String,
    pub username: String,
    pub current_selection: CurrentSelection,
    pub join_room_input: JoinRoomInput,
    pub chat_room_mode: ChatRoomMode,
    pub msg_pipe: Option<tokio::sync::mpsc::Sender<String>>,
    pub abandon: Arc<Mutex<bool>>
}

impl App {
    pub fn new() -> Self {
        App {
            input: String::new(),
            chat_room_record: Arc::new(Mutex::new(VecDeque::new())),
            chat_room_member: Arc::new(Mutex::new(vec![])),
            record_size: 100,
            current_screen: CurrentScreen::Entry,
            editing: false,
            room_id: String::new(),
            username: String::new(),
            current_selection: CurrentSelection::Create,
            join_room_input: JoinRoomInput::Username,
            chat_room_mode: ChatRoomMode::Normal,
            msg_pipe: None,
            abandon: Arc::new(Mutex::new(false))
        }
    }

    pub fn enter_room(&mut self, room_id: String) {
        self.room_id = room_id;
        self.current_screen = CurrentScreen::Chat;
    }

    pub fn join_room(&mut self) {
        self.current_screen = CurrentScreen::Join;
    }
}