
# Stage 1: Build
FROM rust:1.78 AS builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y libpq-dev libclang-dev cmake unzip

RUN wget https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcpu.zip && \
    unzip libtorch-cxx11-abi-shared-with-deps-2.1.0+cpu.zip && \
    mv libtorch /opt/libtorch

# Download models
RUN mkdir -p models

# Prepare whisper model
RUN wget -P models https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# Prepare distilbert-base-uncased-finetuned-sst-2-english
RUN mkdir -p models/distilbert-base-uncased-finetuned-sst-2-english && \
    wget -P models/distilbert-base-uncased-finetuned-sst-2-english "https://huggingface.co/distilbert-base-uncased-finetuned-sst-2-english/resolve/main/config.json" && \
    wget -P models/distilbert-base-uncased-finetuned-sst-2-english "https://huggingface.co/distilbert-base-uncased-finetuned-sst-2-english/resolve/main/vocab.txt" && \
    wget -P models/distilbert-base-uncased-finetuned-sst-2-english "https://huggingface.co/distilbert-base-uncased-finetuned-sst-2-english/resolve/main/rust_model.ot"

# Prepare bert-large-cased-finetuned-conll03-english
RUN mkdir -p models/bert-large-cased-finetuned-conll03-english && \
    wget -P models/bert-large-cased-finetuned-conll03-english https://huggingface.co/dbmdz/bert-large-cased-finetuned-conll03-english/resolve/main/vocab.txt && \
    wget -P models/bert-large-cased-finetuned-conll03-english https://huggingface.co/dbmdz/bert-large-cased-finetuned-conll03-english/resolve/main/config.json && \
    wget -P models/bert-large-cased-finetuned-conll03-english https://huggingface.co/dbmdz/bert-large-cased-finetuned-conll03-english/resolve/main/rust_model.ot

# Prepare bart-large-mnli
RUN mkdir -p models/bart-large-mnli && \
    wget -P models/bart-large-mnli https://huggingface.co/facebook/bart-large-mnli/resolve/main/config.json && \
    wget -P models/bart-large-mnli https://huggingface.co/facebook/bart-large-mnli/resolve/main/vocab.json && \
    wget -P models/bart-large-mnli https://huggingface.co/facebook/bart-large-mnli/resolve/main/merges.txt && \
    wget -P models/bart-large-mnli https://huggingface.co/facebook/bart-large-mnli/resolve/main/rust_model.ot


# Copy source code
COPY . .

# Build the application
RUN cargo build --release --locked

FROM debian:bookworm-slim 

WORKDIR /app

# Install required libraries and PostgreSQL client utilities
RUN apt-get update && apt-get install -y \
    libgomp1 \
    postgresql-client \
    libclang1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/devchallenge /app/devchallenge

# Copy libtorch from the builder stage
COPY --from=builder /opt/libtorch /opt/libtorch

# Copy models from the builder stage
COPY --from=builder /app/models /app/models
COPY --from=builder /app/tmp /app/tmp

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@d621dab2-dfb0-46ab-87a9-8197498d4e29_db:5432/postgres
ENV LIBCLANG_PATH=/usr/lib/llvm-13/lib
ENV LD_LIBRARY_PATH=/opt/libtorch/lib:$LD_LIBRARY_PATH


# Set the command to run the binary directly
CMD ["./devchallenge"]

