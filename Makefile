build-scraper-docker:
	podman build . -f ./scraper/docker/Dockerfile -t metonymy/scraper
upload-scraper-docker: build-scraper-docker
	podman push metonymy/scraper
build-bot-docker:
	podman build . -f ./bot/docker/Dockerfile -t metonymy/bot
upload-bot-docker: build-bot-docker
	podman push metonymy/bot