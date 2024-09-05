use clap::{Parser, ValueEnum, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub operation: Operation,

}

#[derive(Subcommand, Debug)]
pub enum Operation {
    /// Create a room
    Create {
        /// Your username
        username: String
    },
    /// Join a room with room ID
    Join {
        /// Your username
        username: String,
        /// The ID of the room you would like to join
        room_id: String
    }
}