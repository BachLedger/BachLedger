VERSION=v2.3.1_qc

build:
	go mod tidy && go build ./...

ut:
	mkdir -p ../ut
	go test -covermode=count -coverprofile=../ut/chainconf.out ./...
	go tool cover -html=../ut/chainconf.out -o=../ut/chainconf.html

lint:
	golangci-lint run ./...

gomod:
	go get chainmaker.org/chainmaker/common/v2@$(VERSION)
	go get chainmaker.org/chainmaker/logger/v2@$(VERSION)
	go get chainmaker.org/chainmaker/pb-go/v2@$(VERSION)
	go get chainmaker.org/chainmaker/protocol/v2@$(VERSION)
	go get chainmaker.org/chainmaker/utils/v2@$(VERSION)
	go mod tidy

comment:
	gocloc --include-lang=Go --output-type=json --not-match=_test.go . | jq '(.total.comment-.total.files*6)/(.total.code+.total.comment)*100'
