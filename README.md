# ytx
ytx is a command line tool that converts YouTube transcripts into readable articles using LLMs.

# Usage Currently
![ytx live demo](assets/newdemo1.gif)

Second run on same link

![ytx live demo second run](assets/second_run.gif)

## Installation
```
cargo install ytx
```
## Caching
ytx will cache transcripts and generated articles locally. Running ytx on the same video will reuse cached data, making the second run much faster.

## Getting Started
ytx utilizes ollama and its models to take youtube transcripts and make them readable. To be able to use ytx you must have ollama installed.

### Ollama install
[head to ollama to install](https://ollama.com/)

ytx currently has options to use 3 cloud models and 1 local model provided by ollama. To be able to use the cloud models you must have an account with ollama but if you have the local model installed you can select that to be your model of choice without needing an ollama account. Local models speed in generating / rewriting your readable transcripts can vary depending on your hardware. Typically, the cloud models are much quicker.

## Usage
Generate an article from a Youtube video:

```
ytx "https://www.youtube.com/watch?v=VIDEO_ID"
```
Choose a cloud ollama model:
```
ytx "https://www.youtube.com/watch?v=VIDEO_ID" --model kimi-k2"
```
Choose a local ollama model:
```
ytx "https://www.youtube.com/watch?v=VIDEO_ID" --model glm4flash"
```
List saved articles
```
ytx list
```
Open a saved article by index or title:
```
ytx open <index | title>
```

## Example Workflow

Generate an article from a video:
```bash
ytx "https://www.youtube.com/watch?v=VIDEO_ID"
```
List saved articles:
```bash
ytx list
```
Open an article:
```bash
ytx open 1
```
## Features
- Fetch YouTube transcripts
- Convert transcripts into readable articles
- Supports Ollama local and cloud models
- Caches transcripts for faster reruns
- Search transcripts based on title of youtube video
## How It Works
1. Fetch transcript using the ytt crate
2. Send transcript to selected Ollama model
3. Model rewrites transcript into article format
4. Article is saved locally
5. Transcript is cached to speed up future runs

## Roadmap

- [x] search transcripts based on title of youtube video

- [ ] support different file type outputs like pdf or md

- [ ] tui reader built in ytx
