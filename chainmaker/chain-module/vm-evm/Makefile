VERSION=develop
gomod:
	go get chainmaker.org/chainmaker/common/v2@$(VERSION)
	go get chainmaker.org/chainmaker/logger/v2@$(VERSION)
	go get chainmaker.org/chainmaker/pb-go/v2@$(VERSION)
	go get chainmaker.org/chainmaker/protocol/v2@$(VERSION)
	go get chainmaker.org/chainmaker/utils/v2@$(VERSION)
	go mod tidy
	cat go.mod|grep chainmaker