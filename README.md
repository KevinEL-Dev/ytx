# What is ytx?

I want ytx to be a cli tool where you can paste your favorite youtube videos and read them as articles.

## Usage

You can provide a transcript as a text file or you can provide a youtube video link.

Once one is passed, the system currently needs you to have ollama installed on your system and probably an ollama account to use cloud models.

Currently, the cli will return to you an article style of the transcript and hopefully maintaing the youtube creators original message.

## Concerns and future for the project

I want ytx to be a way to take youtube transcripts and get articles from them because I think that would be better than just consuming youtube videos as it feels more intentional. It may just be cope.

Besides that I have ambitious plans for this project.

| Features |
| ------------- |
| Save transcripts for youtube videos that have been already fetched |
| Option to choose ollama models |
| TUI to read article straight from your terminal |

I really want this project to be a free resource. The challenge with this is that its hard to find transcripts that are created for easy readability. So this leads to a difficult issue, what is the best way(free way) to turn these transcripts of people talking into a readable format. The simplest way would be to pass this to an LLM api but  that would mean to use an API key which means billing. Another option was to use some sort of NLP strategy that I won't get into.

I decided that for now the best course of action would be to use an LLM but that just runs on your local machine(duh). I will probably implement the use of your own API key but there is already a solution for that which is comes from the ytt crate in rust that I currently use within this program to retrieve youtube video transcript.

