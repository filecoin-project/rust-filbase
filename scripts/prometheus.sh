#!/usr/bin/env bash

echo "Pulling prometheus docker image..."
docker pull prom/prometheus

echo "Pulling prometheus pushgateway image..."
docker pull prom/pushgateway

echo "Starting pushgateway"
docker stop prometheus-pushgateway
docker run --rm -d -p 9091:9091 --name prometheus-pushgateway prom/pushgateway

echo "Starting prometheus"
docker stop prometheus
docker run --rm -p 9090:9090 --name prometheus -v `pwd`:/prometheus-data prom/prometheus --config.file=/prometheus-data/prometheus.yml
