#!/bin/bash

# Build docker image
docker build -t lichess-bot .

# Replace running lichess-bot container with newly built one
docker kill lichess-bot
docker rm lichess-bot
docker run -d --name=lichess-bot --env-file ~/.env lichess-bot:latest 
docker system prune --force
