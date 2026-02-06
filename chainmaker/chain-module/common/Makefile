VERSION=develop

build:
	go build ./...

mockgen:
	cd msgbus && mockgen -destination ./mock/msg_bus_mock.go -package mock -source ./message_bus.go
ut:
	if [[ `uname` == 'Darwin' ]]; then \
        brew install softhsm; \
    fi; \
    if [[ `uname` == 'Linux' ]]; then \
        yum install softhsm -y; \
    fi; \
    softhsm2-util --init-token --slot 0 --label test --pin 1234 --so-pin 1234; \
	./ut_cover.sh
lint:
	golangci-lint run ./...

gomod:
	echo "nothing"

stringer:
	cd msgbus && stringer -type Topic