# Benchmarking

Using the `benchy` subcommand and feature flag, `filbase` provides a way to benchmark the storage-proofs of filecoin.

## Recording Benchmark Statistics

By default the benchmarks will print the statistics to the console. If you want you can push them to a prometheus backend

You will need the following things running

- [Prometheus](https://prometheus.io)
- [Prometheus PushGateway](https://github.com/prometheus/pushgateway)

and then pass the flag `--push-prometheus` to the benchmark.
In addition you will need to connect your gateway with prometheus as described [here](https://github.com/prometheus/pushgateway#use-it).


## Visualizing Benchmarks

If you want you can setup [Grafana](https://grafana.com) and connect it to your prometheus instance, visualizing the different values.
