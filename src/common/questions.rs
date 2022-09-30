use anyhow::bail;
use rand::prelude::SliceRandom;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleAnswerQuestion {
    pub question: String,
    pub variants: Vec<String>,
    pub correct_answer: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInstance {
    pub id: Uuid,
    pub bound_to: Uuid,
    #[serde(flatten)]
    pub question: SingleAnswerQuestion,
}

#[derive(Debug, Clone)]
pub struct QuizHandler {
    instances: HashMap<Uuid, QuestionInstance>,
    question_folder: PathBuf,
}

impl QuizHandler {
    pub fn new<P: Into<PathBuf>>(question_folder: P) -> Self {
        Self {
            instances: HashMap::new(),
            question_folder: question_folder.into(),
        }
    }

    pub async fn get_all_categories(&mut self) -> anyhow::Result<Vec<String>> {
        Ok(std::fs::read_dir(&self.question_folder)?
            .map(|entry| {
                entry
                    .expect("Invalid entry")
                    .file_name()
                    .into_string()
                    .expect("Invalid OS String")
            })
            .collect::<Vec<String>>())
    }

    pub async fn get_all_from_category(
        &mut self,
        category: String,
    ) -> anyhow::Result<Vec<SingleAnswerQuestion>> {
        let path = self
            .question_folder
            .clone()
            .join(format!("{category}.json"));
        if !path.exists() {
            bail!("Question category {category} does not exist!")
        }
        let mut file = File::open(path).await?;
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf).await?;
        drop(file);
        serde_json::from_str(&buf).map_err(anyhow::Error::from)
    }

    pub async fn get_from_category(
        &mut self,
        user: Uuid,
        category: String,
    ) -> anyhow::Result<QuestionInstance> {
        let question = self
            .get_all_from_category(category.clone())
            .await?
            .choose(&mut OsRng)
            .ok_or(anyhow::Error::msg(format!(
                "No elements in category {category}!"
            )))?
            .clone();
        let id = Uuid::new_v4();
        let instance = QuestionInstance {
            id,
            bound_to: user,
            question,
        };
        self.instances.insert(id, instance.clone());
        Ok(instance)
    }

    pub async fn answer(&mut self, question_id: Uuid, answer: u8) -> anyhow::Result<(bool, u8)> {
        let instance = self
            .instances
            .remove(&question_id)
            .ok_or(anyhow::Error::msg("Invalid question id!"))?;
        return Ok((
            instance.question.correct_answer == answer,
            instance.question.correct_answer,
        ));
    }
}
