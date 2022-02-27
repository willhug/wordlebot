# wordlebot

A discord bot that creates threads for wordle finishers (so you can talk without spoiling things)

Prioritizes creating threads in this order:
- Private threads in the channel
- Public threads in a channel named {original_channel}_solvers
- Public threads in the channel

The big thing is we don't want to spoil people about what the word was.

# Deploy

1. Build the docker container:

```
docker build -t us-west1-docker.pkg.dev/side-341906/wordlebot/wordlebot:<version> -t us-west1-docker.pkg.dev/side-341906/wordlebot/wordlebot:latest .
```

2. Push the container:

```
docker push us-west1-docker.pkg.dev/side-341906/wordlebot/wordlebot:<version> && docker push us-west1-docker.pkg.dev/side-341906/wordlebot/wordlebot:latest
```

Unfortunately we have to push the "latest" tag manually, not sure how to get around that.
