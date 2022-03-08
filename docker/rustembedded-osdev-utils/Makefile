## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2019-2022 Andre Richter <andre.o.richter@gmail.com>

# Reference followed: https://www.docker.com/blog/getting-started-with-docker-for-arm-on-linux

TAG := 2021.12

default: build_local

build_local:
	cp ../../Gemfile .
	docker build                                           \
	    --tag rustembedded/osdev-utils:$(TAG)              \
	    --build-arg VCS_REF=`git rev-parse --short HEAD` .
	rm Gemfile

buildx_push:
	cp ../../Gemfile .
	docker buildx build                                    \
	    --push                                             \
	    --platform linux/arm64/v8,linux/amd64              \
	    --tag rustembedded/osdev-utils:$(TAG)              \
	    --build-arg VCS_REF=`git rev-parse --short HEAD` .
	rm Gemfile

docker_login:
	docker attach rustembedded

docker_run:
	docker run --name rustembedded -i -t rustembedded/osdev-utils

docker_start:
	docker start rustembedded

docker_stop:
	docker stop rustembedded

