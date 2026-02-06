# start node1-node4
```sh
docker-compose -f docker-compose.yaml up -d node1 node2 node3 node4
```

# add org5 TrustRoot and add org5
```sh
cd ../../../../../test/send_proposal_request_tool
./send_proposal_request_tool createContract -w ../wasm/go-counter-1.0.0.wasm -p 7988 --org-id yz-org1.yzchain.org --org-ids yz-org1.yzchain.org,yz-org2.yzchain.org,yz-org3.yzchain.org,yz-org4.yzchain.org
./send_proposal_request_tool invoke  -p 7988 --org-id yz-org1.yzchain.org --org-ids yz-org1.yzchain.org,yz-org2.yzchain.org,yz-org3.yzchain.org,yz-org4.yzchain.org
./send_proposal_request_tool trustRootAdd -p 7988 --org-id yz-org1.yzchain.org --org-ids yz-org1.yzchain.org,yz-org2.yzchain.org,yz-org3.yzchain.org,yz-org4.yzchain.org --trust_root_org_id yz-org5.yzchain.org --trust_root_crt ../../config/yz-org5/certs/ca/yz-org5.yzchain.org/ca.crt
./send_proposal_request_tool chainConfigNodeOrgAdd -p 7988 --org-id yz-org1.yzchain.org --org-ids yz-org1.yzchain.org,yz-org2.yzchain.org,yz-org3.yzchain.org,yz-org4.yzchain.org --nodeOrg_org_id yz-org5.yzchain.org  --nodeOrg_addresses "/ip4/127.0.0.1/tcp/11305/p2p/QmVSCXfPweL1GRSNt8gjcw1YQ2VcCirAtTdLKGkgGKsHqi"
```

# start node5
```sh
docker-compose -f docker-compose.yaml up -d node5
```

# stop network
```sh
docker-compose -f docker-compose.yaml down
```
