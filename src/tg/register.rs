use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::tg::{Command, SignupDialogue};
use teloxide::utils::command::BotCommands;

#[derive(Debug, Clone)]
pub enum DialogueState {
    Start,
    GetUsername
}

impl Default for DialogueState {
    fn default() -> Self {
        DialogueState::Start
    }
}

pub fn schema() -> UpdateHandler<anyhow::Error> {
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![DialogueState::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![DialogueState::GetUsername].endpoint(receive_username_text));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![DialogueState::GetUsername].endpoint(receive_username_callback));

    dialogue::enter::<Update, InMemStorage<DialogueState>, DialogueState, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn help(bot: AutoSend<Bot>, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn cancel(bot: AutoSend<Bot>, msg: Message, dialogue: SignupDialogue) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Регистрация отменена.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_username_callback(
    bot: AutoSend<Bot>,
    q: CallbackQuery,
    dialogue: SignupDialogue,
) -> anyhow::Result<()> {
    if let Some(username) = &q.data {
        bot.send_message(
            dialogue.chat_id(),
            format!("Вы выбрали использовать ваш текущий ник в телеграме: `{}`", username)
        ).await?;

        finish_registration(bot, dialogue.chat_id(), username.to_owned()).await?;

        dialogue.exit().await?;
    }

    Ok(())
}

async fn receive_username_text(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: SignupDialogue,
) -> anyhow::Result<()> {
    match msg.text().map(ToOwned::to_owned) {
        Some(username) => {
            bot.send_message(
                dialogue.chat_id(),
                format!("Вы выбрали ник: `{}`", username)
            ).await?;
            finish_registration(bot, msg.chat.id, username).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Напишите ваш ник.").await?;
        }
    }

    Ok(())
}

fn make_username_keyboard(msg: &Message) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(format!("Использовать `{}`", msg.chat.username().unwrap()), msg.chat.username().unwrap())]])
}

pub async fn start(bot: AutoSend<Bot>, msg: Message, dialogue: SignupDialogue) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Вы регистрируетесь на квест.").await?;
    // todo: actually check if the person is registered

    bot.send_message(msg.chat.id, "Введите предпочитаемый ник.").reply_markup(make_username_keyboard(&msg)).await?;

    dialogue.update(DialogueState::GetUsername).await?;

    Ok(())
}

#[allow(unused_variables)]
async fn finish_registration(bot: AutoSend<Bot>, id: ChatId, username: String) -> anyhow::Result<()> {
    bot.send_message(id, "Регистрация проведена успешно!").await?;

    Ok(())
}