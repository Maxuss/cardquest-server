pub mod register;

use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::dispatching::dialogue::InMemStorage;
use crate::tg::register::{DialogueState, schema};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "Список комманд бота:")]
pub enum Command {
    #[command(description = "Показывает это сообщение")]
    Help,
    #[command(description = "Начинает процесс регистрации")]
    Start,
    #[command(description = "Отменяет процесс регистрации")]
    Cancel,
}

type SignupDialogue = Dialogue<DialogueState, InMemStorage<DialogueState>>;

pub async fn init_tg(tk: String) -> anyhow::Result<()> {
    log::info!("Starting telegram bot...");

    let bot = Bot::new(tk).auto_send();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<DialogueState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
