#!/bin/bash
cd ../tools/yzc/
./yzc parallel invoke \
--hosts=192.168.80.128:12301  \
--contract-name=T \
--method=N \
--user-crts=./testdata/crypto-config/yz-org1.yzchain.org/user/client1/client1.sign.crt  \
--user-keys=./testdata/crypto-config/yz-org1.yzchain.org/user/client1/client1.sign.key  \
--ca-path=./testdata/crypto-config/yz-org1.yzchain.org/ca/  \
--org-IDs=yz-org1.yzchain.org \
--loopNum=1  \
--printTime=5  \
--threadNum=1 \
--timeout=10000 \
--sleepTime=10 \
--climbTime=5  \
--use-tls=true
