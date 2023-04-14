/*This is a Rust program that uses the Serenity and Lettre crates to implement a Discord bot that performs user verification via email. The bot listens for a specific command ([verify]) in a designated verification channel. When a user issues the command, the bot sends a DM to the user requesting their email address. The user responds with their email address, and the bot sends an email containing a verification code. The bot then sends another DM to the user requesting the verification code. The user responds with the verification code, and if it matches the one sent via email, the bot assigns a "verified" role to the user.

The program defines a Handler struct that implements the EventHandler trait from Serenity. The message method of the Handler struct is responsible for handling user commands and implementing the verification process. The ready method of the Handler struct is called when the bot connects to Discord and simply prints a message to the console indicating that the bot is connected.

The program also defines a generate_verification_code function that generates a random alphanumeric string of length 6 to use as the verification code. The send_verification_email function uses the Lettre crate to send an email containing the verification code to the user's email address.

The program uses the env crate to read the Discord bot token from the environment variable BOT_TOKEN. It also defines two constants: verify_channel_id and verify_role_name. The former is the ID of the verification channel, and the latter is the name of the role that will be assigned to verified users.

The program uses the tokio crate to run the main event loop of the Discord bot asynchronously.*/
use std::env;

use rand::{Rng, distributions::Alphanumeric};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use lettre::{Transport, SmtpTransport, SmtpTransportBuilder, message::{EmailMessage, MultiPart}, smtp::{authentication::Credentials, error::SmtpError}};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id.as_u64() != verify_channel_id {
            return;
        }
        if msg.content.to_lowercase() == "[verify]" {
            let email = match msg.author.dm(&ctx.http, |m| {
                m.content("Please enter your email address:")
            }).await {
                Ok(msg) => {
                    if let Some(email) = msg.content.split_once(' ') {
                        email.1.trim().to_owned()
                    } else {
                        let _ = msg.channel_id.say(&ctx.http, "Invalid email address.").await;
                        return;
                    }
                },
                Err(_) => {
                    let _ = msg.channel_id.say(&ctx.http, "Failed to send a DM to verify your email. Please check your DM settings and try again.").await;
                    return;
                },
            };
            let verification_code = generate_verification_code();
            match send_verification_email(&email, &verification_code).await {
                Ok(_) => {
                    let _ = msg.author.dm(&ctx.http, |m| {
                        m.content(format!("Please enter the verification code sent to {}:", email))
                    }).await;
                },
                Err(e) => {
                    let _ = msg.channel_id.say(&ctx.http, format!("Failed to send a verification code to {}. Error: {:?}", email, e)).await;
                    return;
                },
            };
            let verification_code_from_user = match msg.author.dm(&ctx.http, |m| m.content("")).await {
                Ok(msg) => msg.content.trim().to_owned(),
                Err(_) => {
                    let _ = msg.channel_id.say(&ctx.http, "Failed to receive a verification code. Please check your DM settings and try again.").await;
                    return;
                },
            };
            if verification_code_from_user != verification_code {
                let _ = msg.channel_id.say(&ctx.http, "Invalid verification code. Please try again.").await;
                return;
            }
            let verify_role = match msg.guild_id.unwrap().role_by_name(&ctx.http, verify_role_name).await {
                Some(role) => role,
                None => {
                    let _ = msg.channel_id.say(&ctx.http, format!("Failed to assign the {} role. Please make sure the role exists and the bot has permission to assign roles.", verify_role_name)).await;
                    return;
                },
            };
            if let Err(_) = msg.author.add_role(&ctx.http, verify_role.id).await {
                let _ = msg.channel_id.say(&ctx.http, "Failed to assign the verified role. Please try again later or contact an admin for assistance.").await;
                return;
            }

            let _ = msg.author.dm(&ctx.http, |m| {
                m.content("You have been verified!")
            }).await;
            let _ = msg.channel_id.say(&ctx.http, format!("{} has been verified!", msg.author)).await;
        } else {
            let _ = msg.channel_id.say(&ctx.http, "Invalid command. Please use `[verify]` command in the verification channel.").await;
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name}
  const verify_channel_id: u64 = /* Enter the verification channel ID here */;
const verify_role_name: &str = "verified";

fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    let code: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(6)
        .collect();
    code
}

async fn send_verification_email(email: &str, verification_code: &str) -> Result<(), SmtpError> {
    let email_address = format!("{}{}", email, ALLOWED_DOMAIN);
    let credentials = Credentials::new(SMTP_USERNAME.to_owned(), SMTP_PASSWORD.to_owned());
    let smtp_transport = SmtpTransportBuilder::new(("smtp.gmail.com", 465), SmtpTransport::tls())
        .unwrap()
        .credentials(credentials)
        .build();
    let mut multipart = MultiPart::related().add_alternative("<html><body><h2>Your verification code:</h2><p><strong>".to_owned() + verification_code + "</strong></p></body></html>".into(), "text/html".parse().unwrap());
    multipart.add_attachment(include_bytes!("resources/logo.png").to_vec(), None, "logo.png".parse().unwrap());
    let message = EmailMessage::builder()
        .from(format!("{} <{}>", verify_role_name, SMTP_USERNAME))
        .to(email_address.parse().unwrap())
        .subject("Verification code")
        .multipart(multipart)
        .build()?;
    smtp_transport.send(&message).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}

