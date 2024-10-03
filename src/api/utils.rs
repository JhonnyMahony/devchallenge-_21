use anyhow::Result;
use rust_bert::pipelines::ner::NERModel;
use rust_bert::pipelines::sentiment::{SentimentModel, SentimentPolarity};
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;
use simple_transcribe_rs::transcriber::Transcriber;
use sqlx::PgPool;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

use super::models::{CallReindex, Category};

// Get audio file and write to tmp folder
pub async fn download_audio_file(audio_url: &str) -> Result<Uuid> {
    let response = reqwest::get(audio_url).await?;
    let bytes = response.bytes().await?;
    let path = Uuid::new_v4();
    let mut file = File::create(format!("./tmp/{}", path))?;
    file.write_all(&bytes)?;
    Ok(path)
}

// Helper function to transcribe audio using simple_transcribe
pub async fn transcribe_audio(path: String, trans: &Transcriber) -> String {
    let result = trans.transcribe(&path, None).unwrap();
    let text = result.get_text();
    text.to_string()
}

pub async fn emotional_tone(
    text: String,
    sentiment_classifier: &SentimentModel,
) -> Result<Option<String>> {
    let output = sentiment_classifier.predict(&[text.as_str()]); // Pass the vector of &str
    let sentiment = output.into_iter().next().unwrap();
    let emotional_tone = match sentiment.polarity {
        SentimentPolarity::Positive => {
            if sentiment.score > 0.999 {
                "Positive"
            } else {
                "Neutral" // Consider lower positive scores as "Neutral"
            }
        }
        SentimentPolarity::Negative => {
            if sentiment.score > 0.999 {
                "Angry"
            } else if sentiment.score < 0.9 {
                "Negative"
            } else {
                "Neutral"
            }
        }
    };

    Ok(Some(emotional_tone.to_string()))
}

pub async fn name_and_locations(
    text: String,
    ner_model: &NERModel,
) -> Result<(Option<Vec<String>>, Option<Vec<String>>)> {
    let output = ner_model.predict(&[text]);
    // Initialize vectors to hold name and location entities.
    let mut names: Vec<String> = Vec::new();
    let mut locations: Vec<String> = Vec::new();

    // Iterate over the output and categorize entities based on their label (e.g., 'PER' for persons, 'LOC' for locations).
    for entity in output.iter().flatten() {
        match entity.label.as_str() {
            "I-PER" => names.push(entity.clone().word), // "PER" typically represents persons in NER.
            "I-LOC" => locations.push(entity.clone().word), // "LOC" represents locations.
            _ => {}                                     // Ignore other entity types.
        }
    }

    // Return None if no names or locations were found.
    let names_opt = if names.is_empty() { None } else { Some(names) };
    let locations_opt = if locations.is_empty() {
        None
    } else {
        Some(locations)
    };

    // Return the names and locations.
    Ok((names_opt, locations_opt))
}

pub async fn categories(
    text: String,
    categories: Vec<Category>,
    zero_shot: &ZeroShotClassificationModel,
) -> Result<Vec<String>> {
    let candidate_labels: Vec<String> = categories
        .iter()
        .flat_map(|c| {
            let title_iter = std::iter::once(c.title.clone());
            let points_iter = c.points.clone().unwrap_or_else(|| vec![]).into_iter(); // Handle None case
            title_iter.chain(points_iter) // Combine both title and points
        })
        .collect();

    let output = zero_shot.predict_multilabel(
        &[text.as_str()],
        candidate_labels
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
        None,
        128,
    )?;
    let mut unique_categories = HashSet::new();

    // Iterate over the Vec<Vec<Label>> and filter by score
    for label in &output[0] {
        if label.score > 0.89 {
            // Check if the label matches any category title or points
            if let Some(category_title) = categories.iter().find_map(|category| {
                if category.title == label.text
                    || category
                        .points
                        .as_ref()
                        .map_or(false, |points| points.contains(&label.text))
                {
                    Some(category.title.clone())
                } else {
                    None
                }
            }) {
                // Insert the category title into the HashSet (duplicates are automatically handled)
                unique_categories.insert(category_title);
            }
        }
    }
    // Convert the HashSet back into a Vec for the return value
    Ok(unique_categories.into_iter().collect())
}

pub async fn reindex_calls_for_category(
    pool: &PgPool,
    prev_title: Option<&str>,
    category_title: &str,
    candidate_labels: Vec<String>,
    zero_shot: &ZeroShotClassificationModel,
) -> Result<()> {
    // Fetch all calls, regardless of categories

    let calls = sqlx::query_as::<_, CallReindex>(
        r#"
    SELECT id, text, categories FROM call
    "#,
    )
    .fetch_all(pool)
    .await?;

    // Iterate through the calls and classify them
    for call in calls {
        let prediction = zero_shot.predict_multilabel(
            &[call.text.as_str()],
            candidate_labels
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
            None,
            128,
        )?;

        let mut still_belongs = false;

        // Determine if the call belongs to the category

        for label in &prediction[0] {
            if label.score > 0.89 {
                still_belongs = true;
                break;
            }
        }

        let categories = call.categories.unwrap_or_default();
        if still_belongs {
            // If it now belongs, ensure the category is in the call's categories
            if !categories.contains(&category_title.to_string()) {
                let _result = sqlx::query(
                    r#"
        UPDATE call
        SET categories = array_append(categories, $1)
        WHERE id = $2
        "#,
                )
                .bind(category_title)
                .bind(call.id)
                .execute(pool)
                .await?;

                let _category_title = match prev_title {
                    Some(c) => {
                        let _result = sqlx::query(
                            r#"
        UPDATE call
        SET categories = array_remove(categories, $1)
        WHERE id = $2
        "#,
                        )
                        .bind(c)
                        .bind(call.id)
                        .execute(pool)
                        .await?;
                        c
                    }
                    None => category_title,
                };
            }
        } else {
            let category_title = match prev_title {
                Some(c) => c,
                None => category_title,
            };
            // If it no longer belongs, remove the category from the call's categories
            if categories.contains(&category_title.to_string()) {
                let _result = sqlx::query(
                    r#"
        UPDATE call
        SET categories = array_remove(categories, $1)
        WHERE id = $2
        "#,
                )
                .bind(category_title)
                .bind(call.id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}
