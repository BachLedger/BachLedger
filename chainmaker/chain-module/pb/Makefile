all: pb rpc mockgen

pb:
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative accesscontrol/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative common/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative discovery/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative net/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative store/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative sync/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative txpool/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative consensus/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative consensus/tbft/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative consensus/dpos/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative config/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative syscontract/*.proto
	protoc -I=. --gogofaster_out=:../pb-go --gogofaster_opt=paths=source_relative txfilter/*.proto

rpc:
	protoc -I=. \
		-I=${GOPATH}/pkg/mod/github.com/grpc-ecosystem/grpc-gateway@v1.16.0/third_party/googleapis \
		--gogofaster_out=plugins=grpc:../pb-go \
		--gogofaster_opt=paths=source_relative \
		--grpc-gateway_opt=paths=source_relative \
		--grpc-gateway_out=logtostderr=true:../pb-go \
		--swagger_out=logtostderr=true:../pb-go \
		api/*.proto
	protoc -I=. --gogofaster_out=plugins=grpc:../pb-go --gogofaster_opt=paths=source_relative vm/*.proto
	protoc -I=. --gogofaster_out=plugins=grpc:../pb-go --gogofaster_opt=paths=source_relative archivecenter/*.proto	

pb-dep:
	go get -u google.golang.org/grpc/cmd/protoc-gen-go-grpc
	go get -u github.com/golang/protobuf/protoc-gen-go
	go get -u google.golang.org/grpc
	go install github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway@v1.16.0
	go install github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger@v1.16.0
	go get -u github.com/gogo/protobuf/protoc-gen-gogofaster

mockgen:
	mockgen -destination ../pb-go/mock/archivecenter_mock.go -package mock -source ../pb-go/archivecenter/archivecenter.pb.go
	cd ../pb-go && go get github.com/golang/mock@v1.6.0  && go mod tidy