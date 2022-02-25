use std::env;

use serenity::{
    async_trait,
    model::{channel::{Message, ChannelType}, gateway::Ready},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let content = msg.content.trim();
        if let Some(rest) = content.strip_prefix("Wordle ") {
            let day = rest.split_terminator('\n').collect::<Vec<_>>()[0].split_whitespace().collect::<Vec<_>>()[0];
            let thread_name = format!("Wordle Solvers {}", day);
            let chan = msg.channel_id.to_channel(&ctx.http).await.unwrap();
            let threads = chan.guild().unwrap().guild_id.get_active_threads(&ctx.http).await.unwrap();
            let thread = match threads.threads.iter().find(|t| t.name == thread_name) {
                Some(t) => t.clone(),
                None => {
                        let mut res = msg.channel_id.create_private_thread(&ctx, |f| {
                            f.name(thread_name.clone());
                            f.kind(ChannelType::PrivateThread);
                            f.rate_limit_per_user(0);
                            f
                        }).await;
                        if res.is_err() {
                            println!("Failed to create private thread: {}", res.err().unwrap());
                            res = msg.channel_id.create_private_thread(&ctx, |f| {
                                f.name(thread_name);
                                f.kind(ChannelType::PublicThread);
                                f.rate_limit_per_user(0);
                                f
                            }).await;
                        }
                        res.unwrap()
                }
            };
            thread.say(&ctx, format!("Congrats {}, welcome to the secret club", msg.author.mention())).await.unwrap();
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    let token = env::var("WORDLE_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
