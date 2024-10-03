use anyhow::Result;
use rust_bert::pipelines::common::{ModelResource, ModelType};
use rust_bert::pipelines::ner::NERModel;
use rust_bert::pipelines::sentiment::{SentimentConfig, SentimentModel};
use rust_bert::pipelines::token_classification::TokenClassificationConfig;
use rust_bert::pipelines::zero_shot_classification::{
    ZeroShotClassificationConfig, ZeroShotClassificationModel,
};
use simple_transcribe_rs::transcriber::Transcriber;
use simple_transcribe_rs::{model_handler, transcriber};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub sentiment: SentimentModel,
    pub ner: NERModel,
    pub zero_shot: ZeroShotClassificationModel,
    pub transcriber: Transcriber,
}
impl AppState {
    pub async fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            sentiment: sentiment_model()
                .await
                .expect("sentiment model config error"),
            ner: ner_model().await.expect("ner model config error"),
            zero_shot: zero_shot_model()
                .await
                .expect("zero shot model config error"),
            transcriber: trancriber_model().await,
        }))
    }
}

async fn trancriber_model() -> Transcriber {
    let m = model_handler::ModelHandler::new("small", "models").await;
    transcriber::Transcriber::new(m)
}

async fn sentiment_model() -> Result<SentimentModel> {
    let sentiment_classifier = actix_web::web::block(move || {
        let sentiment_config = SentimentConfig {
            model_type: ModelType::DistilBert,
            model_resource: ModelResource::Torch(Box::from(PathBuf::from(
                "./models/distilbert-base-uncased-finetuned-sst-2-english/rust_model.ot",
            ))),
            config_resource: PathBuf::from(
                "./models/distilbert-base-uncased-finetuned-sst-2-english/config.json",
            )
            .to_path_buf()
            .into(),
            vocab_resource: PathBuf::from(
                "./models/distilbert-base-uncased-finetuned-sst-2-english/vocab.txt",
            )
            .to_path_buf()
            .into(),
            merges_resource: None,
            ..Default::default()
        };
        SentimentModel::new(sentiment_config)
    })
    .await??;
    Ok(sentiment_classifier)
}

async fn ner_model() -> Result<NERModel> {
    let ner_model = actix_web::web::block(move || {
        let ner_config = TokenClassificationConfig {
            model_type: ModelType::Bert,
            model_resource: ModelResource::Torch(Box::from(PathBuf::from(
                "./models/bert-large-cased-finetuned-conll03-english/rust_model.ot",
            ))),
            config_resource: PathBuf::from(
                "./models/bert-large-cased-finetuned-conll03-english/config.json",
            )
            .into(),
            vocab_resource: PathBuf::from(
                "./models/bert-large-cased-finetuned-conll03-english/vocab.txt",
            )
            .into(),
            merges_resource: None, // Not needed for BERT-based models
            ..Default::default()
        };
        NERModel::new(ner_config)
    })
    .await??;
    Ok(ner_model)
}

async fn zero_shot_model() -> Result<ZeroShotClassificationModel> {
    let zero_shot_model = actix_web::web::block(move || {
        let zero_shot_config = ZeroShotClassificationConfig {
            model_resource: ModelResource::Torch(Box::from(PathBuf::from(
                "./models/bart-large-mnli/rust_model.ot",
            ))),
            config_resource: PathBuf::from("./models/bart-large-mnli/config.json").into(),
            vocab_resource: PathBuf::from("./models/bart-large-mnli/vocab.json").into(),
            merges_resource: Some(PathBuf::from("./models/bart-large-mnli/merges.txt").into()),
            ..Default::default()
        };

        ZeroShotClassificationModel::new(zero_shot_config)
    })
    .await??;
    Ok(zero_shot_model)
}
