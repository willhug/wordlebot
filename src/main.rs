use std::env;

use serenity::{
    async_trait,
    model::{channel::{Message, ChannelType}, gateway::Ready, guild::PremiumTier},
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
            let guild_chan = chan.guild().unwrap();
            let threads = guild_chan.guild_id.get_active_threads(&ctx.http).await.unwrap();
            let thread = match threads.threads.iter().find(|t| t.name == thread_name) {
                Some(t) => t.clone(),
                None => {
                        let guild = msg.guild_id.unwrap().to_partial_guild(&ctx).await.unwrap();
                        let thread_type = match guild.premium_tier {
                            PremiumTier::Tier3 | PremiumTier::Tier2 => ChannelType::PrivateThread,
                            _ => ChannelType::PublicThread,
                        };
                        let chan_id = match thread_type {
                            ChannelType::PublicThread => {
                                match guild.channels(&ctx).await.unwrap().values().find(|c| c.name == format!("{}_solvers",  guild_chan.name)) {
                                    Some(chan) => chan.id,
                                    None => msg.channel_id,
                                }
                            },
                            ChannelType::PrivateThread => msg.channel_id,
                            _ => unreachable!(),
                        };
                        chan_id.create_private_thread(&ctx, |f| {
                            f.name(thread_name.clone());
                            f.kind(thread_type);
                            f.rate_limit_per_user(0);
                            f
                        }).await.unwrap()
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
