#!/bin/bash
docker build -t r_docker .
docker run -dp 8080:3030 --rm --name r_docker1 r_docker