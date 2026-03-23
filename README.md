# ytx
ytx is a command line utility that generates articles from youtube videos. ytx will fetch youtube transcripts and turn them into readable articles while maintaing the youtube creators message.

# Usage Currently
![ytx live demo](assets/ytx_live_demo.gif)

## Getting Started
ytx utilizes ollama and its models to take youtube transcripts and make them readable. To be able to use ytx you must have ollama installed.

ytx currently has options to use 3 cloud models and 1 local model provided by ollama. To be able to use the cloud models you must have an account with ollama but if you have the local model installed you can select that to be your model of choice without needing an ollama account. Local models speed in generating / rewriting your readable transcripts can vary in depending on your hardware. Typically, the cloud models are much quicker.

## Usage

You can provide a provide a youtube video link.

## Concerns and future for the project
Besides that I have ambitious plans for this project.

| Current Features |
| ------------- |
| Save transcripts for youtube videos that have been already fetched |
| Option to choose ollama models |

I really want this project to be a free resource. The challenge with this is that its hard to find transcripts that are created for easy readability. So this leads to a difficult issue, what is the best way(free way) to turn these transcripts of people talking into a readable format. The simplest way would be to pass this to an LLM api but  that would mean to use an API key which means billing. Another option was to use some sort of NLP strategy that I won't get into.

I decided that for now the best course of action would be to use an LLM but that just runs on your local machine(duh). I will probably implement the use of your own API key but there is already a solution for that which is comes from the ytt crate in rust that I currently use within this program to retrieve youtube video transcript.

