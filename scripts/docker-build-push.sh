#!/usr/bin/env bash
set -e
HASH=$(git rev-parse --verify HEAD)
docker build -t film-sqipper:$HASH .
docker tag film-sqipper:$HASH arranf/film-sqipper:$HASH
docker push arranf/film-sqipper:$HASH