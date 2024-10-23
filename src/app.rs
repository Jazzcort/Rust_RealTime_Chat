use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum CurrentScreen {
    Entry,
    Create,
    CreatePassword,
    Join,
    RoomSelect,
    PasswordCheck,
    Chat,
    Exiting,
}
#[derive(Debug)]
pub enum CurrentSelection {
    Create,
    Join,
}
#[derive(Debug)]
pub enum JoinRoomInput {
    Username,
    RoomId,
}
#[derive(Debug)]
pub enum CreateRoomInput {
    Username,
    RoomName,
}
#[derive(Debug)]
pub enum ChatRoomMode {
    Input,
    Normal,
}
#[derive(Debug)]
pub enum CreateRoomError {
    InvalidRoomNameChar,
    InvalidUsernameChar,
    InvalidUsernameLength,
    InvalidRoomNameLength,
    ServerError,
    InvalidPasswordChar,
}
#[derive(Debug)]
pub enum JoinRoomError {
    InvalidUsername,
    InvalidUsernameLength,
    RoomIdLengthError,
    RoomNotFound,
    GetRoomListFailed,
    WrongPassword,
    ZeroRooms,
}

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub chat_room_record: Arc<Mutex<VecDeque<String>>>,
    pub chat_room_member: Arc<Mutex<Vec<String>>>,
    pub record_size: u32,
    pub current_screen: CurrentScreen,
    pub exiting: bool,
    pub room_id: String,
    pub room_lst: Vec<(String, String, bool)>,
    pub room_idx: usize,
    pub username: String,
    pub current_selection: CurrentSelection,
    pub join_room_input: JoinRoomInput,
    pub chat_room_mode: ChatRoomMode,
    pub msg_pipe: Option<tokio::sync::mpsc::Sender<String>>,
    pub abandon: Arc<Mutex<bool>>,
    pub create_room_error: Option<CreateRoomError>,
    pub join_room_error: Option<JoinRoomError>,
    pub room_name: String,
    pub password: String,
    pub password_prompt: bool,
    pub create_room_input: CreateRoomInput,
    pub check_passwork: String,
}

impl App {
    pub fn new() -> Self {
        App {
            input: String::new(),
            chat_room_record: Arc::new(Mutex::new(VecDeque::new())),
            chat_room_member: Arc::new(Mutex::new(vec![])),
            record_size: 100,
            current_screen: CurrentScreen::Entry,
            exiting: false,
            room_id: String::new(),
            room_lst: vec![],
            room_idx: 0,
            username: String::new(),
            current_selection: CurrentSelection::Create,
            join_room_input: JoinRoomInput::Username,
            chat_room_mode: ChatRoomMode::Normal,
            msg_pipe: None,
            abandon: Arc::new(Mutex::new(false)),
            create_room_error: None,
            join_room_error: None,
            room_name: String::new(),
            password: String::new(),
            password_prompt: false,
            create_room_input: CreateRoomInput::Username,
            check_passwork: String::new(),
        }
    }

    pub fn enter_room(&mut self, room_id: String) {
        self.room_id = room_id;
        self.current_screen = CurrentScreen::Chat;
    }

    pub fn reinitialize(&mut self) {
        self.input = String::new();
        self.chat_room_record = Arc::new(Mutex::new(VecDeque::new()));
        self.chat_room_member = Arc::new(Mutex::new(vec![]));
        self.record_size = 100;
        self.current_screen = CurrentScreen::Entry;
        self.exiting = false;
        self.room_id = String::new();
        self.username = String::new();
        self.current_selection = CurrentSelection::Create;
        self.join_room_input = JoinRoomInput::Username;
        self.chat_room_mode = ChatRoomMode::Normal;
        self.msg_pipe = None;
        self.abandon = Arc::new(Mutex::new(false));
        self.create_room_error = None;
        self.join_room_error = None;
        self.room_idx = 0;
        self.room_lst = vec![];
        self.room_name = String::new();
        self.password = String::new();
        self.password_prompt = false;
        self.create_room_input = CreateRoomInput::Username;
        self.check_passwork = String::new();
    }
}
