use serenity::prelude::Context;
use serenity::model::prelude::Message;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;
use tokio::time::Instant;

#[command]
#[aliases(p)]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Instant::now();
    let mut ping = msg.channel_id.say(&ctx.http, format!("pong! (`pinging...`)", )).await.unwrap();
    let _ = ping.edit(&ctx.http, |m| m.content(format!("pong! (`{:?}`)", now.elapsed()))).await;


    Ok(())
}