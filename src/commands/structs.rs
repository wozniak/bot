use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;
use serenity::client::bridge::voice::ClientVoiceManager;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub struct MusicQueue;

impl TypeMapKey for MusicQueue {
    type Value = Arc<Mutex<HashMap<GuildId, Vec<String>>>>;
}

pub struct MusicSkip;
impl TypeMapKey for MusicSkip {
    type Value = Arc<Mutex<HashMap<GuildId, bool>>>;
}