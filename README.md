# wordlebot

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
