use crate::common::models::UserRegStage;
use crate::tg::{Command, SignupDialogue};
use sqlx::PgPool;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, UpdateHandler};
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::utils::command::BotCommands;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum DialogueState {
    Start,
    GetUsername { id: Uuid, card_hash: String },
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
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::Register(token)].endpoint(register)),
        )
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message().branch(command_handler).branch(
        case![DialogueState::GetUsername { id, card_hash }].endpoint(receive_username_text),
    );

    let callback_query_handler = Update::filter_callback_query().branch(
        case![DialogueState::GetUsername { id, card_hash }].endpoint(receive_username_callback),
    );

    dialogue::enter::<Update, InMemStorage<DialogueState>, DialogueState, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn help(bot: AutoSend<Bot>, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: AutoSend<Bot>, msg: Message, dialogue: SignupDialogue) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Регистрация отменена.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn receive_username_callback(
    bot: AutoSend<Bot>,
    q: CallbackQuery,
    dialogue: SignupDialogue,
    pool: PgPool,
    (id, card_hash): (Uuid, String),
) -> anyhow::Result<()> {
    if let Some(username) = &q.data {
        if user_exists(&pool, username).await? {
            bot.send_message(dialogue.chat_id(), format!("Пользователь с ником `{}` уже существует\\!\nПожалуйста, выберите другой ник\\.", username)).parse_mode(ParseMode::MarkdownV2).await?;
            return Ok(());
        }
        bot.send_message(
            dialogue.chat_id(),
            format!(
                "Вы выбрали использовать ваш текущий ник в телеграме: `{}`",
                username
            ),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

        finish_registration(
            bot,
            dialogue.chat_id(),
            pool,
            username.to_owned(),
            id,
            card_hash,
        )
        .await?;

        dialogue.exit().await?;
    }

    Ok(())
}

async fn receive_username_text(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: SignupDialogue,
    pool: PgPool,
    (id, card_hash): (Uuid, String),
) -> anyhow::Result<()> {
    match msg.text().map(ToOwned::to_owned) {
        Some(username) => {
            if user_exists(&pool, &username).await? {
                bot.send_message(dialogue.chat_id(), format!("Пользователь с ником `{}` уже существует\\!\nПожалуйста, выберите другой ник\\.", username)).parse_mode(ParseMode::MarkdownV2).await?;
                return Ok(());
            }
            bot.send_message(
                dialogue.chat_id(),
                format!("Вы выбрали ник: `{}`", username),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
            finish_registration(bot, msg.chat.id, pool, username, id, card_hash).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Напишите ваш ник.").await?;
        }
    }

    Ok(())
}

fn make_username_keyboard(msg: &Message) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        format!("Использовать {}", msg.chat.username().unwrap()),
        msg.chat.username().unwrap(),
    )]])
}

pub async fn start(bot: AutoSend<Bot>, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id,
                     "Этот бот позволяет вам регистрироваться на квест\\.\nНачните процесс регистрации командой `/register <токен>`,\n заменив `<token>`на ваш токен регистрации\\."
    ).parse_mode(ParseMode::MarkdownV2).await?;

    Ok(())
}

pub async fn register(
    bot: AutoSend<Bot>,
    msg: Message,
    dialogue: SignupDialogue,
    token: String,
    pool: PgPool,
) -> anyhow::Result<()> {
    if token.len() != 8 {
        bot.send_message(msg.chat.id, "Неверный токен регистрации!")
            .await?;
        return Ok(());
    }
    let stage = if let Some(stage) =
        sqlx::query_as::<_, UserRegStage>("SELECT * FROM users_reg WHERE starts_with(hash, $1)")
            .bind(token)
            .fetch_optional(&pool)
            .await?
    {
        stage
    } else {
        bot.send_message(msg.chat.id, "Неверный токен для регистрации!")
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Вы начинаете регистрацию на квест.")
        .await?;

    let deleted = sqlx::query("DELETE FROM users_reg WHERE starts_with(hash, $1)")
        .bind(&stage.hash)
        .execute(&pool)
        .await?;

    if deleted.rows_affected() != 1 {
        log::warn!(
            "Invalid amount of rows affected for delete operation, expected 1 but got {}",
            deleted.rows_affected()
        )
    }

    bot.send_message(msg.chat.id, "Введите предпочитаемый ник.")
        .reply_markup(make_username_keyboard(&msg))
        .await?;

    dialogue
        .update(DialogueState::GetUsername {
            id: stage.id,
            card_hash: stage.hash,
        })
        .await?;

    Ok(())
}

async fn user_exists(pool: &PgPool, username: &String) -> anyhow::Result<bool> {
    if let Some(_) = sqlx::query("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[allow(unused_variables)]
async fn finish_registration(
    bot: AutoSend<Bot>,
    id: ChatId,
    pool: PgPool,
    username: String,
    uuid: Uuid,
    card_hash: String,
) -> anyhow::Result<()> {
    let rows = sqlx::query("INSERT INTO users VALUES($1, $2, $3)")
        .bind(card_hash)
        .bind(uuid)
        .bind(username)
        .execute(&pool)
        .await?;
    if rows.rows_affected() < 1 {
        bot.send_message(id, "Не удалось провести регистрацию!")
            .await?;
        return Ok(());
    }

    bot.send_message(id, "Регистрация проведена успешно!")
        .await?;
    Ok(())
}
