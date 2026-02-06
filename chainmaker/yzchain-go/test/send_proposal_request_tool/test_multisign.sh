#/bin/bash

server_ip="127.0.0.1"
server_port=12301
tools_path=/Users/cao/yzchain-go/test/send_proposal_request_tool
project_path=/Users/cao/yzchain-go


${tools_path}/send_proposal_request_tool multiSignReq  \
--sys-contract-name="CONTRACT_MANAGE"   \
--sys-method="INIT_CONTRACT"   \
--pairs="[{\"key\":\"CONTRACT_NAME\",\"value\":\"contract107\",\"IsFile\":false},{\"key\":\"CONTRACT_VERSION\",\"value\":\"1.0\",\"IsFile\":false},{\"key\":\"CONTRACT_BYTECODE\",\"value\":\"/Users/cao/yzchain-go/test/wasm/rust-counter-2.0.0.wasm\",\"IsFile\":true},{\"key\":\"CONTRACT_RUNTIME_TYPE\",\"value\":\"WASMER\",\"IsFile\":false}]"  \
--ip=127.0.0.1  \
--port=12301  \
--user-key=${project_path}/config/yz-org1/certs/user/client1/client1.sign.key  \
--user-crt=${project_path}/config/yz-org1/certs/user/client1/client1.sign.crt  \
--ca-path=${project_path}/config/yz-org1/certs/ca/yz-org1.yzchain.org  \
--use-tls=true  \
--chain-id=chain1  \
--org-id=yz-org1.yzchain.org  \
--org-ids=yz-org1.yzchain.org  \
--admin-sign-crts=${project_path}/config/yz-org1/certs/user/admin1/admin1.sign.crt  \
--admin-sign-keys=${project_path}/config/yz-org1/certs/user/admin1/admin1.sign.key


