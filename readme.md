# Social Media News Bot

A simple Bot that scrapes websites and publishes tweets on [BlueSky](https://bsky.app/) and [Mastodon](https://joinmastodon.org).

It's built with [Rust](https://www.rust-lang.org/) and [Redis](https://redis.io/) and can be extended to include 
LLM support for content summarization, suggestions and other features.

Demo:

![demo bluesky](./docs/demo_bluesky.jpg)

---

![demo mastodon](./docs/demo_mastodon.png)

## Architecture

![architecture diagram](./docs/architecture_diagram.drawio.png)

The architecture is composed of the following elements:

1. The Scrapper

It scrapes data from one or more websites and publishes a JSON on **Redis Streams**.

It is configured via CLI arguments 

```bash
Usage: scrapper [OPTIONS] --redis-connection-string <redis_connection_string> --redis-stream-name <REDIS_STREAM_NAME>

Options:
  -r, --redis-connection-string <redis_connection_string>
          Redis host
  -t, --redis-stream-name <REDIS_STREAM_NAME>
          Redis stream name
  -s, --scrape-interval-minutes <SCRAPE_INTERVAL_MINUTES>
          The scraping interval in minutes [default: 60]
  -h, --help
          Print help
  -V, --version
          Print version
```

2. Redis

Redis is a key-value store with lots of features. It has been chosen to keep 
things simple and due to its powerful features and flexibility[1].

3. BlueSky / Mastodon Bot

The bot reads data from Redis Streams and publishes it to the selected platform.

```shell
Social media posting bot.

Usage: bot --redis-connection-string <redis_connection_string> --redis-stream-name <REDIS_STREAM_NAME> --redis-consumer-group <REDIS_CONSUMER_GROUP> --redis-consumer-name <REDIS_CONSUMER_NAME> <COMMAND>

Commands:
  bluesky   Post on bluesky platform
  mastodon  Post on Mastodon, the FediVerse
  help      Print this message or the help of the given subcommand(s)

Options:
  -r, --redis-connection-string <redis_connection_string>  Redis host
  -t, --redis-stream-name <REDIS_STREAM_NAME>              Redis stream name
  -c, --redis-consumer-group <REDIS_CONSUMER_GROUP>        Redis consumer group name
  -n, --redis-consumer-name <REDIS_CONSUMER_NAME>          The current consumer name
  -h, --help                                               Print help
  -V, --version                                            Print version
```

[1] - https://redis.io/about/

## Development

The rust version used for development is `rustc 1.92.0 (ded5c06cf 2025-12-08)`.

### Running the tests

The tests can be run with `cargo test`.

Tests use testcontainers by default to spin the necessary dependencies, currently only Redis is required.
Alternatively you can use `REDIS_TESTS_URL` in order to make the tests run agains an existing Redis service.

**Note**: For Podman users you will need to enable the podman socket. Otherwise, testcontainers won't be able to 
create new containers[1].

```shell
systemctl enable podman.socket
podman system service --time=0 &
```

## Running locally

You can build the project and run each binary separately. A `docker-compose.yml` file is provided in order to start
necessary dependencies.

[1] - https://podman-desktop.io/tutorial/testcontainers-with-podman

--- 


