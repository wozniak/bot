use serenity::{
    prelude::*,
    voice,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        macros::{
            command,
        },
    },
};

use crate::commands::structs::*;
use serenity::static_assertions::_core::time::Duration;
use yt_api::ApiKey;
use yt_api::search::{SearchList, ItemType};
use std::time::Instant;

#[command]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let youtube_token: String;
    {
        let data = ctx.data.read().await;
        youtube_token = String::from(data.get::<Config>().unwrap()["tokens"]["youtube"].as_str().unwrap());
    }

    let search_term = msg.content.split(" ").skip(1).collect::<Vec<_>>().join(" ");

    let key = ApiKey::new(youtube_token);
    let search = SearchList::new(key)
        .q(&search_term)
        .item_type(ItemType::Video);

    let list = search.perform().await.unwrap();

    let song = &list.items[0];
    let title = song.snippet.title.as_ref().unwrap();

    // queues song
    {
        let mut data = ctx.data.write().await;
        let mut queue = data.get_mut::<MusicQueue>().unwrap().lock().await;

        match queue.get_mut(&msg.guild_id.unwrap()) {
            Some(v) => v.insert(0, title.clone()),
            None => { queue.insert(msg.guild_id.unwrap(), vec![title.clone()]); },
        }
    }

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description(format!("queued '{}'", &title))
        })
    }).await.unwrap();

    loop {
        let data = ctx.data.read().await;
        let map = data.get::<MusicQueue>().unwrap().lock().await;
        let queue = map.get(&msg.guild_id.unwrap()).unwrap();
        if queue.last().unwrap() == title {
            break;
        }

        drop(map);
        drop(data);

        tokio::time::delay_for(Duration::new(2, 0)).await;
    }

    println!("it is time! starting!");


    let data = ctx.data.read().await;
    let mut manager = data.get::<VoiceManager>().unwrap().lock().await;
    let handler = manager.join(msg.guild_id.unwrap(),
                           msg.guild(&ctx.cache)
                               .await.unwrap()
                               .voice_states
                               .get(&msg.author.id).unwrap()
                               .channel_id.unwrap()
    ).unwrap();

    println!("getting source");
    let source = voice::ytdl(&format!("http://youtube.com/watch?v={}", song.id.video_id.as_ref().unwrap())).await.unwrap();

    println!("playing...");
    let audio = handler.play_only(source);

    drop(manager);
    drop(data);

    loop {
        {
            let lock = audio.lock().await;
            if lock.finished { break; }
            drop(lock);

            let mut data = ctx.data.write().await;
            let mut skip_map = data.get_mut::<MusicSkip>().unwrap().lock().await;

            if skip_map.get(&msg.guild_id.unwrap()) == Some(&true) {
                skip_map.insert(msg.guild_id.unwrap(), false);
                break;
            }
        }
        tokio::time::delay_for(Duration::new(2, 0)).await;
    }

    println!("song is done lol");

    let data = ctx.data.write().await;
    let mut queue = data.get::<MusicQueue>().unwrap().lock().await;
    queue.get_mut(&msg.guild_id.unwrap()).unwrap().pop();

    Ok(())
}

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let l = ctx.data.read().await.get::<MusicQueue>().cloned().unwrap();
    let x = l.lock().await;
    let mut songs = match x.get(&msg.guild_id.unwrap()) {
        Some(v) => v.clone(),
        None => Vec::<String>::new(),
    };

    songs.reverse();

    if songs.len() == 0 {
        msg.channel_id.send_message(&ctx.http, |m| m.content("queue is empty")).await.unwrap();
    } else {
        msg.channel_id.send_message(&ctx.http, |f| {
            f.embed(move |mut e| {
                e = e.title("queue");
                for song in 0..songs.len() {
                    if song == 0 {
                        e = e.field("current song", songs[song].clone(), false);
                    } else {
                        e = e.field(format!("#{}", song), songs[song].clone(), false);
                    }
                }
                e
            })
        }).await.unwrap();
    }

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let mut vote = msg.channel_id.send_message(&ctx.http, |m| m.content("skip current song?")).await.unwrap();

    vote.react(&ctx.http, ReactionType::from('👌')).await.unwrap();

    let votes_needed: usize;
    {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let channel = guild.voice_states.get(&msg.author.id).unwrap().channel_id.unwrap().to_channel(&ctx.http).await.unwrap();
        let guild_channel = channel.guild().unwrap();
        votes_needed = guild_channel.members(&ctx.cache).await.unwrap().len() / 2 + 1;
    }

    let now = Instant::now();
    loop {
        let vote = ctx.http.get_message(vote.channel_id.0, vote.id.0).await.unwrap();
        tokio::time::delay_for(Duration::from_secs(1)).await;
        println!("{:?}", vote.reactions);
        if vote.reactions[0].count >= votes_needed as u64 {
            break;
        } else {
            if now.elapsed() > Duration::from_secs(30) {
                msg.channel_id.say(&ctx.http, "vote didn't pass, ran out of time").await.unwrap();
                return Ok(());
            }
        }
    }

    {
        let mut data = ctx.data.write().await;

        let manager_lock = data.get::<VoiceManager>().cloned().unwrap();
        let mut manager = manager_lock.lock().await;

        let guild = msg.guild(&ctx.cache).await.unwrap();
        let channel = guild.voice_states.get(&msg.author.id).unwrap();

        let handler = manager.join(msg.guild_id.unwrap(), channel.channel_id.unwrap()).unwrap();
        handler.stop();

        let mut skip_map = data.get_mut::<MusicSkip>().unwrap().lock().await;
        skip_map.insert(msg.guild_id.unwrap(), true);
    }

    vote.edit(&ctx.http, |m| m.content("skipped a song")).await.unwrap();

    Ok(())

}