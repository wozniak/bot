use serenity::{
    prelude::*,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        Args,
        macros::command,
    },
};

use rosu::Osu;
use crate::commands::structs::Config;

#[command]
async fn osu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let username = args.raw().collect::<Vec<_>>().join(" ");

    // get the user
    let data = ctx.data.read().await;
    let osu_token = data.get::<Config>().unwrap()["tokens"]["osu"].as_str().unwrap();

    let osu = Osu::new(osu_token);

    let user_request = rosu::backend::UserRequest::with_username(username).unwrap();
    let user = match user_request.queue_single(&osu).await.unwrap() {
        Some(u) => u,
        None => {
            msg.channel_id.send_message(&ctx.http, |m| m.content("no such user")).await.unwrap();
            return Ok(());
        }
    };

    // embed go brr
    msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.author(|a| {
            a.name(user.username.clone())
                .icon_url(format!("http://s.ppy.sh/a/{}", user.user_id))
                .url(format!("https://osu.ppy.sh/users/{}", user.user_id))
        })
            .field("PP",        user.pp_raw,    true)
            .field("country",   user.country,   true)
            .field("level",     user.level,     true)
            .field("join date", user.join_date, true)
            .field("SS count",  user.count_ss,  true)
            .field("S count",   user.count_s,   true)
    })).await.unwrap();

    Ok(())
}