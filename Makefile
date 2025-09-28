build-scraper-docker:
	podman build . -f ./scrapper/docker/Dockerfile -t metonymy/scrapper
upload-scraper-docker: build-scraper-docker
	podman push metonymy/scrapper
build-bot-docker:
	podman build . -f ./bot/docker/Dockerfile -t metonymy/bot
upload-bot-docker: build-bot-docker
	podman push metonymy/bot