use serenity::{
    prelude::*,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        macros::command,
        Args,
    },
};

#[command]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let count = match args.single::<u64>() {
        Ok(u) => u,
        Err(_) => {
            let _ = msg.channel_id.say(&ctx.http, "must have a number of messages to purge").await;
            return Ok(());
        }
    };

    let messages = msg.channel_id.messages(&ctx.http, |m| m.limit(count + 1)).await.unwrap();

    let _ = msg.channel_id.delete_messages(&ctx.http, messages.into_iter().map(|m| m.id)).await;
    let _ = msg.channel_id.say(&ctx.http, format!("purged {} messages", count)).await;

    Ok(())
}