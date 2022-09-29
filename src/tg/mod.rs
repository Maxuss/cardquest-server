pub mod register;

use crate::tg::register::{schema, DialogueState};
use sqlx::PgPool;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "Список комманд бота:")]
pub enum Command {
    #[command(description = "Показывает это сообщение")]
    Help,
    #[command(description = "Показывает основную информацию про этого бота.")]
    Start,
    #[command(description = "Начинает процесс регистрации. Берет токен регистрации как аргумент.")]
    Register(String),
    #[command(description = "Отменяет процесс регистрации")]
    Cancel,
}

type SignupDialogue = Dialogue<DialogueState, InMemStorage<DialogueState>>;

pub async fn init_tg(tk: String, pool: PgPool) -> anyhow::Result<()> {
    log::info!("Starting telegram bot...");

    let bot = Bot::new(tk).auto_send();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<DialogueState>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
