use serde::{Serialize, Deserialize};


pub enum ClientState {
    Ready,
    Updating,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    SkipUpdate,
    DoUpdate,       // Need hash
    HashMatched,
    HashMismatched, // Need file
}
