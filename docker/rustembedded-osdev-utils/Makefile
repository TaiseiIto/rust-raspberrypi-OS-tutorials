## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2019-2021 Andre Richter <andre.o.richter@gmail.com>

default: docker_build

docker_build:
	cp ../../Gemfile .
	docker build -t rustembedded/osdev-utils \
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

