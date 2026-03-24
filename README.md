# ytx
ytx is a command line tool that converts YouTube transcripts into readable articles using LLMs.

# Usage Currently
![ytx live demo](assets/newdemo1.gif)

Second run on same link

![ytx live demo second run](assets/second_run.gif)

## Installation
cargo install ytx

## On second run
ytx will cache transcripts that have all ready been generated to improve speed.
## Getting Started
ytx utilizes ollama and its models to take youtube transcripts and make them readable. To be able to use ytx you must have ollama installed.

ytx currently has options to use 3 cloud models and 1 local model provided by ollama. To be able to use the cloud models you must have an account with ollama but if you have the local model installed you can select that to be your model of choice without needing an ollama account. Local models speed in generating / rewriting your readable transcripts can vary in depending on your hardware. Typically, the cloud models are much quicker.

## Usage
You can provide a provide a youtube video link.

```
ytx -l "https://www.youtube.com/watch?v=VIDEO_ID"
```
Choose a cloud ollama model:
```
ytx -l "https://www.youtube.com/watch?v=VIDEO_ID --model kimi-k2"
```
Choose a local ollama model:
```
ytx -l "https://www.youtube.com/watch?v=VIDEO_ID --model glm4flash"
```
## Features
- Fetch YouTube transcripts
- Convert transcripts into readable articles
- Supports Ollama local and cloud models
- Caches transcripts for faster reruns
## How It Works
1. Fetch transcript using the ytt crate
2. Send transcript to selected Ollama model
3. Model rewrites transcript into article format
4. Article is saved locally
5. Transcript is cached to speed up future runs

## Future Features

- [ ] search transcripts based on title of youtube video

- [ ] support different file type outputs like pdf or md

- [ ] tui reader built in ytx
