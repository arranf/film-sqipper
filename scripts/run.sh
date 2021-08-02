#!/usr/bin/env bash
set -e
cat ./example.json | amqpcat --producer --uri=amqp://guest:guest@localhost:5672 --queue sqip.create