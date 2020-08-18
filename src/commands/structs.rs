use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serenity::client::bridge::voice::ClientVoiceManager;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub struct MusicQueue;

impl TypeMapKey for MusicQueue {
    type Value = Arc<RwLock<HashMap<GuildId, Vec<String>>>>;
}