PLATFORM=$(shell uname -m)
DATETIME=$(shell date "+%Y%m%d%H%M%S")
VERSION=develop

yzchain-cryptogen:
	@cd src && go mod tidy && GOPATH=${GOPATH} go build -o ../bin/yzchain-cryptogen
	@mkdir -p ./release
	@rm -rf ./tmp/yzchain-cryptogen/
	@mkdir -p ./tmp/yzchain-cryptogen/
	@mkdir ./tmp/yzchain-cryptogen/bin
	@mkdir ./tmp/yzchain-cryptogen/config
	@cp bin/yzchain-cryptogen ./tmp/yzchain-cryptogen/bin
	@cp -r config/crypto_config_template.yml ./tmp/yzchain-cryptogen/config
	@cd ./tmp;tar -zcvf yzchain-cryptogen-$(VERSION).$(DATETIME).$(PLATFORM).tar.gz yzchain-cryptogen; mv yzchain-cryptogen-$(VERSION).$(DATETIME).$(PLATFORM).tar.gz ../release
	@rm -rf ./tmp/

clean:
	@rm -rf ./bin
