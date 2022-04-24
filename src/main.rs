use std::env;

mod detector;
mod words;
use detector::{calculate_word_possibilities, parse_words_list};
use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::{ChannelType, Message},
        gateway::Ready,
        guild::PremiumTier,
        misc::Mention,
    },
    prelude::*,
};

#[tokio::main]
async fn main() {
    let token = env::var("WORDLE_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let content = msg.content.trim();
        if let Some(query) = extract_wordle_stats_query(content) {
            let mut rows = match parse_words_list(query) {
                Ok(rows) => rows,
                Err(e) => {
                    dbg!(e);
                    msg.reply(
                        ctx,
                        "Weird, couldn't parse your message, I need 5 letters per row.",
                    )
                    .await
                    .unwrap();
                    return;
                }
            };
            let res = match calculate_word_possibilities(&mut rows) {
                Ok(res) => res,
                Err(e) => {
                    dbg!(e);
                    msg.reply(ctx, "Weird, something went wrong running, not sure what")
                        .await
                        .unwrap();
                    return;
                }
            };
            let mut result = String::new();
            for (i, row) in rows.iter().enumerate() {
                let row_str = format!(
                    "`{}` - {} words: (`{}`)\n",
                    row.iter().collect::<String>(),
                    res[i].2,
                    res[i].0.join("`, `"),
                );
                result.push_str(&row_str);
            }
            msg.reply(ctx, result).await.unwrap();
            return;
        }
        if let Some((name, day, result, body)) = extract_wordlelike_data(content) {
            let thread_name = format!("{} Solvers {}", name, day);
            let chan = msg.channel_id.to_channel(&ctx.http).await.unwrap();
            let guild_chan = chan.guild().unwrap();
            let threads = guild_chan
                .guild_id
                .get_active_threads(&ctx.http)
                .await
                .unwrap();
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
                            match guild
                                .channels(&ctx)
                                .await
                                .unwrap()
                                .values()
                                .find(|c| c.name == format!("{}_solvers", guild_chan.name))
                            {
                                Some(chan) => chan.id,
                                None => msg.channel_id,
                            }
                        }
                        ChannelType::PrivateThread => msg.channel_id,
                        _ => unreachable!(),
                    };
                    chan_id
                        .create_private_thread(&ctx, |f| {
                            f.name(thread_name.clone());
                            f.kind(thread_type);
                            f.rate_limit_per_user(0);
                            f
                        })
                        .await
                        .unwrap()
                }
            };
            thread
                .say(
                    &ctx,
                    get_welcome_message(name, msg.author.mention(), result, body),
                )
                .await
                .unwrap();
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn extract_wordle_stats_query(content: &str) -> Option<&str> {
    lazy_static! {
        static ref WORDLE_STATS_REG: Regex = Regex::new(r"!wordlestats((?s).*)").unwrap();
    }
    let captures = WORDLE_STATS_REG.captures(content)?;
    let result = captures.get(1)?.as_str().trim();
    Some(result)
}

fn extract_wordlelike_data(content: &str) -> Option<(&str, u32, &str, &str)> {
    lazy_static! {
        static ref WORDLELIKE_REG: Regex = Regex::new(r"^#?(?:Daily )?([a-zA-Z]*) #?(\d+) ?([\dX])?(?:/6)?\*?((.|\n)*)?$").unwrap();
    }
    let captures = WORDLELIKE_REG.captures(content)?;
    let name = captures.get(1)?.as_str();
    let day = captures.get(2)?.as_str().parse::<u32>().ok()?;
    let result = match captures.get(3) {
        Some(m) => m.as_str(),
        None => "",
    };
    let body = match captures.get(4) {
        Some(body) => body.as_str().trim(),
        None => "",
    };
    Some((name, day, result, body))
}

fn get_welcome_message(typ: &str, author: Mention, result: &str, body: &str) -> String {
    let (suffix_msg, result) = match typ {
        "Wordle" | "Tradle" => (
            match result {
                "1" => "WTFFF?!?!?!",
                "2" => "Master! You're a master!",
                "5" => "Just made it!",
                "6" => "Phew! That was a close one!",
                "X" => "Nutz! Better luck next time!",
                _ => "Nice! You got it!",
            },
            format!("{}/6", result),
        ),
        _ => ("Nice!", result.to_string()),
    };
    format!(
        "Welcome to the secret {} club {}\n{}\n{} {}",
        typ, author, body, result, suffix_msg
    )
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_wordle_regex() {
        assert_eq!(extract_wordlelike_data("Wordle 1 1/6").unwrap(), ("Wordle", 1, "1", ""));
        assert_eq!(
            extract_wordlelike_data("Wordle 200 3/6*").unwrap(),
            ("Wordle", 200, "3", "")
        );
        assert_eq!(extract_wordlelike_data("Wordle 9 X/6").unwrap(), ("Wordle", 9, "X", ""));
        assert_eq!(
            extract_wordlelike_data(
                "Wordle 229 6/6
⬛🟨🟨⬛⬛
🟩⬛⬛⬛🟨
🟩🟩⬛⬛⬛
🟩🟩⬛⬛⬛
🟩🟩⬛⬛🟨
🟩🟩🟩🟩🟩
"
            )
            .unwrap(),
            (
                "Wordle",
                229,
                "6",
                "⬛🟨🟨⬛⬛
🟩⬛⬛⬛🟨
🟩🟩⬛⬛⬛
🟩🟩⬛⬛⬛
🟩🟩⬛⬛🟨
🟩🟩🟩🟩🟩"
            )
        );
    }

    #[test]
    fn test_heardle_regex() {
        assert_eq!(extract_wordlelike_data("#Heardle #16").unwrap(), ("Heardle", 16, "", ""));
        assert_eq!(extract_wordlelike_data("Heardle 16").unwrap(), ("Heardle", 16, "", ""));
        assert_eq!(
            extract_wordlelike_data(
                "#Heardle #16

🔈🟥⬛️⬛️🟩⬜️⬜️"
            )
            .unwrap(),
            ("Heardle", 16, "", "🔈🟥⬛️⬛️🟩⬜️⬜️")
        );
    }

    #[test]
    fn test_tradle_regex() {
        assert_eq!(extract_wordlelike_data("#Tradle #7 1/6").unwrap(), ("Tradle", 7, "1", ""));
        assert_eq!(extract_wordlelike_data("Tradle 7 1/6").unwrap(), ("Tradle", 7, "1", ""));
        assert_eq!(
            extract_wordlelike_data(
                "#Tradle #7 1/6
🟩🟩🟩🟩🟩
https://oec.world/en/tradle"
            )
            .unwrap(),
            (
                "Tradle",
                7,
                "1",
                "🟩🟩🟩🟩🟩
https://oec.world/en/tradle"
            )
        );
    }

    #[test]
    fn test_quordle_regex() {
        assert_eq!(
            extract_wordlelike_data(
                "Daily Quordle #50
5️⃣4️⃣
6️⃣7️⃣"
            )
            .unwrap(),
            (
                "Quordle",
                50,
                "",
                "5️⃣4️⃣
6️⃣7️⃣"
            )
        );
        assert_eq!(
            extract_wordlelike_data(
                "Daily Quordle 50
5️⃣4️⃣
6️⃣7️⃣"
            )
            .unwrap(),
            (
                "Quordle",
                50,
                "",
                "5️⃣4️⃣
6️⃣7️⃣"
            )
        );
    }

    #[test]
    fn test_octordle_regex() {
        assert_eq!(
            extract_wordlelike_data(
                "Daily Octordle 50
6️⃣🔟
4️⃣9️⃣
7️⃣🕛
5️⃣🕚"
            )
            .unwrap(),
            (
                "Octordle",
                50,
                "",
                "6️⃣🔟
4️⃣9️⃣
7️⃣🕛
5️⃣🕚"
            )
        );
        assert_eq!(
            extract_wordlelike_data(
                "Daily Octordle #50
6️⃣🔟
4️⃣9️⃣
7️⃣🕛
5️⃣🕚"
            )
            .unwrap(),
            (
                "Octordle",
                50,
                "",
                "6️⃣🔟
4️⃣9️⃣
7️⃣🕛
5️⃣🕚"
            )
        );
    }

    #[test]
    fn test_wordle_stats() {
        assert_eq!(
            extract_wordle_stats_query(
                "!wordlestats
train
weigh
slide
oxide"
            )
            .unwrap(),
            "train
weigh
slide
oxide"
        );
    }
}
