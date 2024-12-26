build-docker:
	docker build . -f ./scrapper/docker/Dockerfile -t metonymy/scrapper
upload-docker: build-docker
	docker push metonymy/scrapper