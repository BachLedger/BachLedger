ifeq ($(OS),Windows_NT)
    PLATFORM="Windows"
else
    ifeq ($(shell uname),Darwin)
        PLATFORM="MacOS"
    else
        PLATFORM="Linux"
    endif
endif

VERSION=v2.3.2
DATETIME=$(shell date "+%Y%m%d%H%M%S")
GIT_BRANCH = $(shell git rev-parse --abbrev-ref HEAD)
GIT_COMMIT = $(shell git log --pretty=format:'%h' -n 1)

LOCALCONF_HOME=yzchain.org/yzchain-go/module/blockchain
GOLDFLAGS += -X "${LOCALCONF_HOME}.CurrentVersion=${VERSION}"
GOLDFLAGS += -X "${LOCALCONF_HOME}.BuildDateTime=${DATETIME}"
GOLDFLAGS += -X "${LOCALCONF_HOME}.GitBranch=${GIT_BRANCH}"
GOLDFLAGS += -X "${LOCALCONF_HOME}.GitCommit=${GIT_COMMIT}"

yzchain:
    ifeq ($(PLATFORM),"Windows")
		@echo "build for windows"
		@cd main && go mod tidy && go build -ldflags '${GOLDFLAGS}' -o ../bin/yzchain.exe
    else
		@echo "build for linux or mac"
		@cd main && go mod tidy && go build -ldflags '${GOLDFLAGS}' -o ../bin/yzchain
    endif

yzchain-vendor:
	@cd main && go build -mod=vendor -o ../bin/yzchain

package:
	@cd main && go mod tidy && GOPATH=${GOPATH} go build -ldflags '${GOLDFLAGS}' -o ../bin/yzchain
	@mkdir -p ./release
	@rm -rf ./tmp/yzchain/
	@mkdir -p ./tmp/yzchain/
	@mkdir ./tmp/yzchain/bin
	@mkdir ./tmp/yzchain/config
	@mkdir ./tmp/yzchain/log
	@cp bin/yzchain ./tmp/yzchain/bin
	@cp -r config ./tmp/yzchain/
	@cd ./tmp;tar -zcvf yzchain-$(VERSION).$(DATETIME).$(PLATFORM).tar.gz yzchain; mv yzchain-$(VERSION).$(DATETIME).$(PLATFORM).tar.gz ../release
	@rm -rf ./tmp/

compile:
	@cd main && go mod tidy && go build -ldflags '${GOLDFLAGS}' -o ../bin/yzchain

yzc:
	@cd tools/yzc && go mod tidy && go build -ldflags '${GOLDFLAGS}' -o ../../bin/yzc

send-tool:
	cd test/send_proposal_request_tool && go build -o ../../bin/send_proposal_request_tool

scanner:
	@cd tools/scanner && GOPATH=${GOPATH} go build -o ../../bin/scanner

dep:
	@go get golang.org/x/tools/cmd/stringer

generate:
	go generate ./...

docker-build:
	rm -rf build/ data/ log/ bin/
	docker build -t yzchain -f ./DOCKER/Dockerfile .
	docker tag yzchain yzchain:${VERSION}

docker-build-dev: yzchain yzc
	docker build -t yzchain -f ./DOCKER/dev.Dockerfile .
	docker tag yzchain yzchain:${VERSION}

docker-compose-start: docker-compose-stop
	docker-compose up -d

docker-compose-stop:
	docker-compose down

ut:
	cd scripts && ./ut_cover.sh

lint:
	cd main && golangci-lint run ./...
	cd module/accesscontrol && golangci-lint run .
	cd module/blockchain && golangci-lint run .
	cd module/core && golangci-lint run ./...
	cd module/consensus && golangci-lint run ./...
	cd module/net && golangci-lint run ./...
	cd module/rpcserver && golangci-lint run ./...
	cd module/snapshot && golangci-lint run ./...
	cd module/subscriber && golangci-lint run ./...
	cd module/sync && golangci-lint run ./...
	cd module/txfilter && golangci-lint run ./...
	golangci-lint run ./tools/yzc/...
	cd tools/scanner && golangci-lint run ./...

gomod:
	cd scripts && ./gomod_update.sh

test-deploy:
	cd scripts/test/ && ./quick_deploy.sh

sql-qta:
	echo "clear environment"
	cd test/chain2 && ./stop.sh
	cd test/chain2 && ./clean.sh
	echo "start new sql-qta test"
	cd test/chain2 && ./build.sh
	cd test/chain2 && ./start.sh
	cd test/scenario0_native && python3 chain2.py
	cd test/scenario1_evm && python3 chain2.py
	cd test/scenario2_rust && python3 chain2.py
	cd test/scenario4_wasmer_sql && python3 chain2.py
	cd test/chain2 && ./stop.sh
	cd test/chain2 && ./clean.sh

qta: cert-qta pub-qta docker-qta

cert-qta:
	echo "clear environment"
	cd test/chain1 && ./stop.sh
	cd test/chain1 && ./clean.sh
	echo "start new qta test"
	cd test/chain1 && ./build.sh
	cd test/chain1 && ./start.sh
	cd test/scenario0_native && python3 chain1.py
	cd test/scenario1_evm && python3 chain1.py
	cd test/scenario2_rust && python3 chain1.py
	cd test/chain1 && ./stop.sh
	cd test/chain1 && ./clean.sh

pub-qta:
	echo "clear environment"
	cd test/chain3 && ./stop.sh
	cd test/chain3 && ./clean.sh
	echo "start new qta test"
	cd test/chain3 && ./build.sh
	cd test/chain3 && ./start.sh
	cd test/scenario0_native && python3 chain3.py
	cd test/scenario1_evm && python3 chain3.py
	#cd test/scenario2_rust && python3 chain3.py  #Rust合约不能启用Gas
	cd test/chain3 && ./stop.sh
	cd test/chain3 && ./clean.sh

docker-qta:
	echo "clear environment"
	cd test/chain1 && ./stop.sh
	cd test/chain1 && ./clean.sh
	echo "start new docker-qta test"
	cd scripts/docker && ./build-dockergo.sh
	cd test/chain1 && ./build.sh
	cd test/chain1 && ./docker-start.sh
	cd test/chain1 && ./start.sh
	cd test/scenario3_dockergo && python3 chain1.py
	cd test/chain1 && ./stop.sh
	cd test/chain1 && ./clean.sh
	docker rm -f  `docker ps -aq -f name=ci-chain1`

generate-commit-id:
	echo `git log -1 | awk 'NR==1'` > commit_id

generate-vendor:
	sed -i "s/go 1.16/go 1.18/g" go.mod
	go mod tidy
	rm -rf vendor
	go mod vendor
	# 注意：执行此方法前需要切换common项目到对应分支或commit
	# 密码学 gmssl 相关
	cd ../common && git checkout v2.3.2_qc && git pull
	cp -a ../common/opencrypto/gmssl/gmssl/include ./vendor/yzchain.org/yzchain/common/v2/opencrypto/gmssl/gmssl/
	cp -a ../common/opencrypto/gmssl/gmssl/lib ./vendor/yzchain.org/yzchain/common/v2/opencrypto/gmssl/gmssl/lib/




package-source-code: generate-commit-id generate-vendor
	cd .. ; tar -czvf yzchain-go-source.tar.gz --exclude=yzchain-go/.git  --exclude=yzchain-go/test  --exclude=yzchain-go/bin  --exclude=yzchain-go/build  --exclude=yzchain-go/data  --exclude=yzchain-go/tools/yzc1  --exclude=yzchain-go/log --exclude=.DS_Store yzchain-go

vendor-build:
	#ln -s vendor/yzchain.org/yzchain/vm-wasmer/v2/wasmer-go/wasmer-go/packaged/lib/linux-aarch64/libwasmer.so /usr/lib
	cd main && go build -mod=vendor -ldflags '${GOLDFLAGS}' -o ../bin/yzchain

vendor-build-yzc:
	cd tools/yzc &&  go build -mod=vendor -ldflags '${GOLDFLAGS}' -o ../../bin/yzchain

docker-build-vendor: generate-commit-id generate-vendor
	rm -rf build/ data/ log/ bin/
	docker build -t yzchain -f ./DOCKER/vendor.Dockerfile .
	docker tag yzchain yzchain:${VERSION}

clean:
	rm -rf bin/yzchain