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
use crate::commands::structs::{Bank, Config};

#[command]
async fn pay(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let payer = &msg.author;
    
    let recipiant = match args.single::<UserId>() {
        Ok(u) => u,
        Err(e) => {
            let _ = msg.channel_id.say(&ctx.http, format!("error: {:?}", e)).await;
            return Ok(());
        }
    }.to_user(&ctx.http).await.unwrap();
    
    let amount = match args.single::<usize>() {
        Ok(u) => u,
        Err(e) => {
            let _ = msg.channel_id.say(&ctx.http, format!("error: {:?}", e)).await;
            return Ok(());
        }
    };

    {
        let mut data = ctx.data.write().await;
        let start = data.get::<Config>().unwrap()["economy"]["starter"].as_integer().unwrap() as usize;
        let mut bank = data.get_mut::<Bank>().unwrap().lock().await;
        match bank.get_mut(&payer.id) {
            Some(u) => {
                if *u < amount {
                    let _ = msg.channel_id.say(&ctx.http, "you don't have enough for that").await;
                    return Ok(());
                }
                *u -= amount
            },
            None => {
                if start < amount {
                    let _ = msg.channel_id.say(&ctx.http, "you don't have enough for that").await;
                    return Ok(());
                }
                let _ = bank.insert(payer.id, start - amount);
            },
        }

        match bank.get_mut(&recipiant.id) {
            Some(u) => *u += amount,
            None => { let _ = bank.insert(recipiant.id, amount + start); },
        }
    }

    let _ = msg.channel_id.say(&ctx.http, format!("payed {} {}u", recipiant.mention(), amount)).await;

    Ok(())
}

#[command]
async fn bal(ctx: &Context, msg: &Message) -> CommandResult {
    let target: UserId = match msg.mentions.len() {
        0 => msg.author.id,
        _ => msg.mentions[0].id,
    };

    let data = ctx.data.read().await;
    let bank = data.get::<Bank>().unwrap();
    let money = match bank.lock().await.get(&target) {
        Some(u) => *u,
        None => data.get::<Config>().unwrap()["economy"]["starter"].as_integer().unwrap() as usize,
    };
    let _ = msg.channel_id.say(&ctx.http, money).await;

    Ok(())
}

#[command]
async fn gamble(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let risker = &msg.author;
    let money = match args.single::<usize>() {
        Ok(u) => u,
        Err(_) => {
            let _ = msg.channel_id.say(&ctx.http, "you need an amount of money to gamble").await;
            return Ok(());
        }
    };

    let data = ctx.data.write().await;
    let mut bank = data.get::<Bank>().unwrap().lock().await;
    let config = data.get::<Config>().unwrap();

    match bank.get(&risker.id) {
        Some(_) => {},
        None => { let _ = bank.insert(risker.id, config["economy"]["starter"].as_integer().unwrap() as usize); },
    }

    if bank.get(&risker.id).unwrap() < &money {
        let _ = msg.channel_id.say(&ctx.http, "you don't have enough money to gamble").await;
        return Ok(());
    }

    let won: bool = rand::random();
    match won {
        true  => *bank.get_mut(&risker.id).unwrap() += money,
        false => *bank.get_mut(&risker.id).unwrap() -= money,
    }

    let result_string = match won {
        true => "won",
        false => "lost"
    };

    let _ = msg.channel_id.say(&ctx.http,
    format!("you gambled and **{}** `{}u`. current balance is `{}u`",
            result_string,
            money,
            bank.get(&risker.id).unwrap()
    )).await;

    Ok(())
}