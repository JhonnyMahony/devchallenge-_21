# Ministry of Foreign Affairs Telephone Conversation Analysis API

## Description

The Ministry of Foreign Affairs requires a system to process and analyze a large volume of telephone conversations. The objective is to extract important information from these conversations, including:

1. **Names** and **locations** mentioned in the conversation.
2. **Categories** the conversation belongs to (topics of discussion).
3. The **emotional tone** of the conversation.

The processed data will form a structured dataset for future analysis and insights, helping specialists evaluate call quality based on these extracted topics.

## Features

### Call Processing API
- **Submit Audio Files**: Users can submit telephone conversations via a URL (supports `.wav` and `.mp3` formats).
- **Download and Transcription**: The API downloads and transcribes the audio content.
- **Key Information Extraction**: Extracts key details such as caller name and location, if available.
- **Emotional Tone Analysis**: Determines the emotional tone of the conversation (Neutral, Positive, Negative, Angry).
- **Categories**: Each conversation is assigned one or more relevant categories for classification.

### Categories API (CRUD)
Categories represent the topics discussed during a conversation. The API supports:
- **Create**: Add new categories (topics of conversation).
- **Read**: View existing categories and their assigned conversations.
- **Update**: Modify existing categories and reassign conversations if necessary.
- **Delete**: Remove a category, ensuring that any associated conversations are reassigned or managed appropriately.

## Future Plans

This API will serve as the foundation for developing a multi-platform system, including:
- **Web Application**: Built using Rust and Yew for the frontend.
- **Desktop Application**: Built using Tauri and Yew or React.
- **Mobile Applications**: Built using Tauri for Android and iOS.
  
All platforms will connect to the same API, providing a unified system for specialists to analyze calls and generate reports.

## Technology Stack

- **Rust**: The backend is built using Rust for performance and safety.
- **Actix Web**: Used for building the web server and handling asynchronous requests.
- **Rust-BERT**: Used for sentiment and emotional tone analysis of conversations.
- **SQLx**: For database interactions.
- **Tokio**: Asynchronous programming to handle file downloads, transcription, and processing.

## Setup

### Prerequisites
- Rust (https://www.rust-lang.org/)
- PostgreSQL (or any supported database)
- Actix Web
- Rust-BERT
- libtorch (for Rust-BERT)
- FFmpeg (for audio conversion)
  
### Installation

1. Run docker:
```bash
docker-compose up --build
```
2. Run tests:
```bash
    cargo test
```
