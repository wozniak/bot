use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;
use serenity::client::bridge::voice::ClientVoiceManager;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;
use toml::Value;
use serenity::model::id::UserId;

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

pub struct Config;
impl TypeMapKey for Config {
    type Value = Arc<Value>;
}

pub struct Bank;
impl TypeMapKey for Bank {
    type Value = Arc<Mutex<HashMap<UserId, usize>>>;
}