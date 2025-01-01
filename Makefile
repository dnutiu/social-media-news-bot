build-scraper-docker:
	docker build . -f ./scrapper/docker/Dockerfile -t metonymy/scrapper
upload-scraper-docker: build-scraper-docker
	docker push metonymy/scrapper
build-bot-docker:
	docker build . -f ./bot/docker/Dockerfile -t metonymy/bot
upload-bot-docker: build-bot-docker
	docker push metonymy/bot