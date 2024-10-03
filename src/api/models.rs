use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

// Model for category data
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i32,
    pub title: String,
    pub points: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateCategory {
    pub title: String,
    pub points: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct UpdateCategory {
    pub title: Option<String>,
    pub points: Option<Vec<String>>,
}

#[derive(Serialize, FromRow)]
pub struct CallId {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct CallReindex {
    pub id: Uuid,
    pub text: String,
    pub categories: Option<Vec<String>>,
}

// Model for call data
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Call {
    pub id: Uuid,
    pub name: Option<String>,
    pub location: Option<String>,
    pub emotional_tone: Option<String>,
    pub text: String,
    pub categories: Option<Vec<String>>,
}
