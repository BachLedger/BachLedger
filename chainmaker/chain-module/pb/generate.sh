#!/bin/bash

cd ..
rm -rf pb-go
git clone -b v2.3.1.2_qc git@git.code.tencent.com:ChainMaker/pb-go.git
cd pb-go
git checkout -B "$1"
git pull origin "$1"
git clean -d -fx
cd "$3"
set -e
make all
cd ../pb-go
go mod tidy
git add .
git commit -am "$2"
git push --set-upstream origin "$1"
