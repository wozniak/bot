use serenity::{
    prelude::*,
    voice,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        macros::command,
    },
};

use crate::commands::structs::*;
use serenity::static_assertions::_core::time::Duration;

#[command]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let url: String = msg.content.split(" ").skip(1).collect::<Vec<_>>().join(" ");

    // queues song
    {
        let mut data = ctx.data.write().await;
        let mut queue = data.get_mut::<MusicQueue>().unwrap().write().await;

        match queue.get_mut(&msg.guild_id.unwrap()) {
            Some(v) => v.insert(0, url.clone()),
            None => { queue.insert(msg.guild_id.unwrap(), vec![url.clone()]); },
        }
    }

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description(format!("queued '{}'", &url))
        })
    }).await.unwrap();

    loop {
        let data = ctx.data.read().await;
        let map = data.get::<MusicQueue>().unwrap().read().await;
        let queue = map.get(&msg.guild_id.unwrap()).unwrap();
        if queue.last().unwrap() == &url {
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
    let source = voice::ytdl_search(&url).await.unwrap();

    println!("playing...");
    let _ = handler.play_only(source);

    //loop {
    //    {
    //        let lock = audio.lock().await;
    //        if !lock.playing { break; }
    //    }
    //    tokio::time::delay_for(Duration::new(2, 0));
    //}

    //let data = ctx.data.write().await;
    //let queue = data.get::<MusicQueue>().unwrap().write().await;
    //queue.get(&msg.guild_id.unwrap()).unwrap().pop();

    Ok(())
}

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let l = ctx.data.read().await.get::<MusicQueue>().cloned().unwrap();
    let x = l.read().await;
    let mut songs = x.get(&msg.guild_id.unwrap()).unwrap().clone();
    songs.reverse();

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

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let data = ctx.data.read().await;

        let manager_lock = data.get::<VoiceManager>().cloned().unwrap();
        let mut manager = manager_lock.lock().await;

        let guild = msg.guild(&ctx.cache).await.unwrap();
        let channel = guild.voice_states.get(&msg.author.id).unwrap();

        let handler = manager.join(msg.guild_id.unwrap(), channel.channel_id.unwrap()).unwrap();
        handler.stop();
    }

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description("music stopped")
        })
    }).await.unwrap();

    let mut data = ctx.data.write().await;

    let mut queue = data.get_mut::<MusicQueue>().unwrap().write().await;
    let guild_queue = queue.get_mut(&msg.guild_id.unwrap()).unwrap();
    guild_queue.pop();

    Ok(())

}