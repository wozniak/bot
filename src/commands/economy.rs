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
            None => { let _ = bank.insert(recipiant.id, amount); },
        }
    }

    let _ = msg.channel_id.say(&ctx.http, format!("payed {} {}", recipiant.mention(), amount)).await;

    Ok(())
}

#[command]
async fn bal(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let bank = data.get::<Bank>().unwrap();
    let money = match bank.lock().await.get(&msg.author.id) {
        Some(u) => *u,
        None => data.get::<Config>().unwrap()["economy"]["starter"].as_integer().unwrap() as usize,
    };
    let _ = msg.channel_id.say(&ctx.http, money).await;

    Ok(())
}

#[command]
async fn gamble(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let risker = &msg.author;
    let money = match args.single::<isize>() {
        Ok(u) => u,
        Err(_) => {
            let _ = msg.channel_id.say(&ctx.http, "need a sum of money to gamble").await;
            return Ok(());
        }
    };

    let data = ctx.data.write().await;
    let mut bank = data.get::<Bank>().unwrap().lock().await;

    if match bank.get(&risker.id) { Some(u) => *u as isize, None => data.get::<Config>().unwrap()["economy"]["starter"].as_integer().unwrap() as isize } > money {
        let _ = msg.channel_id.say(&ctx.http, "you don't have enough money to gamble").await;
    }

    let amount: isize;
    let won = rand::random::<bool>();
    match won {
        true => amount = money,
        false => amount = -money,
    }

    match bank.get_mut(&risker.id) {
        Some(u) => *u = (*u as isize + money) as usize,
        None => {
            bank.insert(risker.id, (data.get::<Config>().unwrap()["economy"]["starter"].as_integer().unwrap() as isize + amount) as usize);
        }
    };

    let result_string: String;
    if won { result_string = String::from("won") } else { result_string = String::from("lost") }

    let _ = msg.channel_id.say(&ctx.http,
    format!("you gambled and **{}** `{}`, current balance is `{}`",
            result_string,
            money,
            bank.get(&risker.id).unwrap()
    ));


    Ok(())
}