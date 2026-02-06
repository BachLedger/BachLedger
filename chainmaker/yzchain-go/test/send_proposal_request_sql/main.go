/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

/*
sql rust test/wasm/rust-sql-2.0.0.wasm 源码所在目录：chainmaker-contract-sdk-rust v2.0.0_dev src/contract_fact_sql.rs
sql tinygo go-test/wasm/sql-2.0.0.wasm 源码所在目录：chainmaker-contract-sdk-tinygo develop demo/main_fact_sql.go
*/
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"chainmaker.org/chainmaker-go/module/accesscontrol"
	"chainmaker.org/chainmaker-go/test/common"
	"chainmaker.org/chainmaker/common/v2/ca"
	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	acPb "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	apiPb "chainmaker.org/chainmaker/pb-go/v2/api"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/consensus"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/utils/v2"
	"github.com/gogo/protobuf/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

const (
	logTempMarshalPayLoadFailed = "marshal payload failed, %s"
	logTempSendTx               = "send tx resp: code:%d, msg:%s, payload:%+v\n"
)

const (
	CHAIN1         = "chain1"
	IP             = "localhost"
	Port           = 12351
	certPathPrefix = "../../config"
	userKeyPath    = certPathPrefix + "/crypto-config/yz-org1.yzchain.org/user/client1/client1.sign.key"
	userCrtPath    = certPathPrefix + "/crypto-config/yz-org1.yzchain.org/user/client1/client1.sign.crt"
	orgId          = "yz-org1.yzchain.org"
	prePathFmt     = certPathPrefix + "/crypto-config/yz-org%s.chainmaker.org/user/admin1/"

	IPOrg2          = "localhost"
	PortOrg2        = 12352
	userKeyPathOrg2 = certPathPrefix + "/crypto-config/yz-org2.yzchain.org/user/client1/client1.sign.key"
	userCrtPathOrg2 = certPathPrefix + "/crypto-config/yz-org2.yzchain.org/user/client1/client1.sign.crt"
	orgIdOrg2       = "yz-org2.yzchain.org"
	prePathFmtOrg2  = certPathPrefix + "/crypto-config/yz-org%s.chainmaker.org/user/admin1/"
)

var (
	WasmPath        = ""
	WasmUpgradePath = ""
	contractName    = ""
	runtimeType     = commonPb.RuntimeType_EVM
)

var (
	caPaths    = []string{certPathPrefix + "/crypto-config/yz-org1.yzchain.org/ca"}
	client     apiPb.RpcNodeClient
	clientOrg2 *apiPb.RpcNodeClient
	conn       *grpc.ClientConn
	connOrg2   *grpc.ClientConn
	err        error
)

func init() {

}
func initClientOrg2(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient) (isSolo bool) {
	fmt.Println("====================get chain config===================")
	// 构造Payload
	var pairs []*commonPb.KeyValuePair

	payloadBytes := common.ConstructQueryPayload(syscontract.SystemContract_CHAIN_CONFIG.String(), syscontract.ChainConfigFunction_GET_CHAIN_CONFIG.String(), pairs)

	resp := common.ProposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		CHAIN1, "", payloadBytes, nil)

	result := &configPb.ChainConfig{}
	err = proto.Unmarshal(resp.ContractResult.Result, result)
	fmt.Println("Consensus Type is", result.Consensus.Type)

	isSolo = result.Consensus.Type == consensus.ConsensusType_SOLO
	if isSolo {
		clientOrg2 = client
		return isSolo
	}

	// init client org2
	caPaths = append(caPaths, certPathPrefix+"/crypto-config/yz-org2.yzchain.org/ca")
	connOrg2, err = initGRPCConnectOrg2(true)
	if err != nil {
		panic(err)
	}
	c2 := apiPb.NewRpcNodeClient(connOrg2)
	clientOrg2 = &c2
	return isSolo
}

// vm wasmer 整体功能测试，合约创建、升级、执行、查询、冻结、解冻、吊销、交易区块的查询、链配置信息的查询
func main() {
	common.SetCertPathPrefix(certPathPrefix)

	// init client org1
	conn, err = initGRPCConnect(true)
	if err != nil {
		panic(err)
	}
	defer conn.Close()
	client = apiPb.NewRpcNodeClient(conn)

	file, err := ioutil.ReadFile(userKeyPath)
	if err != nil {
		panic(err)
	}

	sk3, err := asym.PrivateKeyFromPEM(file, nil)
	if err != nil {
		panic(err)
	}
	// init client org2
	if isSolo := initClientOrg2(sk3, &client); !isSolo {
		defer connOrg2.Close()
	}

	// test

	//performanceTest(sk3, &client)
	//testWaitTx(sk3, &client, CHAIN1, "20fa21fcff774cef96bcf6294306caa8d30fb9d27dac4484b5ffceaaf018ef79")

}

func functionalTest(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient) {
	var (
		txId   string
		id     string
		result string
		rs     = make(map[string][]byte, 0)
	)

	fmt.Println("//1) 合约创建", time.Now().Format("2006-01-02 15:04:05"))
	txId = testCreate(sk3, client, CHAIN1)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	fmt.Println("// 2) 执行合约-sql insert", time.Now().Format("2006-01-02 15:04:05"))
	txId = testInvokeSqlInsert(sk3, client, CHAIN1, "11", true)
	txId = testInvokeSqlInsert(sk3, client, CHAIN1, "11", true)

	for i := 0; i < 10; i++ {
		txId = testInvokeSqlInsert(sk3, client, CHAIN1, strconv.Itoa(i), false)
	}
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)
	id = txId

	fmt.Println("// 3) 查询 age11的 id:"+id, time.Now().Format("2006-01-02 15:04:05"))
	_, result = testQuerySqlById(sk3, clientOrg2, CHAIN1, id)
	json.Unmarshal([]byte(result), &rs)
	fmt.Println("testInvokeSqlUpdate query", rs, time.Now().Format("2006-01-02 15:04:05"))
	if string(rs["id"]) != id {
		fmt.Println("result", rs)
		panic("query by id error, id err")
	} else {
		fmt.Println("  【testInvokeSqlInsert】 pass", time.Now().Format("2006-01-02 15:04:05"))
		fmt.Println("  【testQuerySqlById】 pass")
	}

	fmt.Println("// 4) 执行合约-sql update name=长安链chainmaker_update where id="+id, time.Now().Format("2006-01-02 15:04:05"))
	txId = testInvokeSqlUpdate(sk3, client, CHAIN1, id)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	fmt.Println("// 5) 查询 id="+id+" 看name是不是更新成了长安链chainmaker_update：", time.Now().Format("2006-01-02 15:04:05"))
	_, result = testQuerySqlById(sk3, clientOrg2, CHAIN1, id)
	json.Unmarshal([]byte(result), &rs)
	fmt.Println("testInvokeSqlUpdate query", rs)
	if string(rs["name"]) != "长安链chainmaker_update" {
		fmt.Println("result", rs)
		panic("query update result error")
	} else {
		fmt.Println("  【testInvokeSqlUpdate】 pass")
	}

	fmt.Println("// 6) 范围查询 rang age 1~10", time.Now().Format("2006-01-02 15:04:05"))
	testQuerySqlRangAge(sk3, clientOrg2, CHAIN1)

	fmt.Println("// 7) 执行合约-sql delete by id age=11", time.Now().Format("2006-01-02 15:04:05"))
	txId = testInvokeSqlDelete(sk3, client, CHAIN1, id)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	fmt.Println("// 8) 再次查询 id age=11，应该查不到", time.Now().Format("2006-01-02 15:04:05"))
	_, result = testQuerySqlById(sk3, clientOrg2, CHAIN1, id)
	if result != "{}" {
		fmt.Println("result", result)
		panic("查询结果错误")
	} else {
		fmt.Println("  【testInvokeSqlDelete】 pass")
	}
	//// 9) 跨合约调用
	txId = testCrossCall(sk3, client, CHAIN1)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	// 10) 交易回退
	txId = testInvokeSqlInsert(sk3, client, CHAIN1, "2000", true)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)
	id = txId
	for i := 0; i < 3; i++ {
		fmt.Println("试图将id="+id+" 的name改为长安链chainmaker_save_point，但是发生了错误，所以修改不会成功", time.Now().Format("2006-01-02 15:04:05"))
		txId = testInvokeSqlUpdateRollbackDbSavePoint(sk3, client, CHAIN1, id)
		testWaitTx(sk3, clientOrg2, CHAIN1, txId)

		fmt.Println("// 11 再次查询age=2000的这条数据，如果name被更新了，那么说明savepoint Rollback失败了", time.Now().Format("2006-01-02 15:04:05"))
		_, result = testQuerySqlById(sk3, clientOrg2, CHAIN1, id)
		rs = make(map[string][]byte, 0)
		json.Unmarshal([]byte(result), &rs)
		fmt.Println("testInvokeSqlUpdateRollbackDbSavePoint query", rs)
		if string(rs["name"]) == "chainmaker_save_point" {
			panic("testInvokeSqlUpdateRollbackDbSavePoint test 【fail】 query by id error, age err")
		} else if string(rs["name"]) == "长安链chainmaker" {
			fmt.Println("  【testInvokeSqlUpdateRollbackDbSavePoint】 pass")
		} else {
			panic("error result")
		}
	}

	// 9) 升级合约
	txId = testUpgrade(sk3, client, CHAIN1)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	// 10) 升级合约后执行插入
	txId = testInvokeSqlInsert(sk3, client, CHAIN1, "100000", true)
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	_, result = testQuerySqlById(sk3, clientOrg2, CHAIN1, txId)
	rs = make(map[string][]byte, 0)
	json.Unmarshal([]byte(result), &rs)
	fmt.Println("testInvokeSqlInsert query", rs)
	if string(rs["age"]) != "100000" {
		panic("query by id error, age err")
	} else {
		fmt.Println("  【testUpgrade】 pass", time.Now().Format("2006-01-02 15:04:05"))
		fmt.Println("  【testInvokeSqlInsert】 pass")
	}

	// 并发测试
	for i := 500; i < 600; i++ {
		txId = testInvokeSqlInsert(sk3, client, CHAIN1, strconv.Itoa(i), false)
	}
	testWaitTx(sk3, clientOrg2, CHAIN1, txId)

	fmt.Println("\nfinal result: ", txId, result, rs, id)
	fmt.Println("test success!!!")
	fmt.Println("test success!!!")
}

func testCreate(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	return common.CreateContract(sk3, client, chainId, contractName, WasmPath, runtimeType)
}

func testUpgrade(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	fmt.Println("============================================================")
	fmt.Println("========================test upgrade========================")
	fmt.Println("============================================================")
	fmt.Println("============================================================")

	resp := common.UpgradeContract(sk3, client, chainId, contractName, WasmUpgradePath, runtimeType)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return resp.TxId
}

func testInvokeSqlInsert(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, age string, print bool) string {
	txId := utils.GetRandTxId()
	if print {
		fmt.Printf("\n============ invoke contract %s[sql_insert] [%s,%s] ============\n", contractName, txId, age)
	}
	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(txId),
		},
		{
			Key:   "age",
			Value: []byte(age),
		},
		{
			Key:   "name",
			Value: []byte("长安链chainmaker"),
		},
		{
			Key:   "id_card_no",
			Value: []byte("510623199202023323"),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_insert",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)
	if print {
		fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	}
	return txId
}

func InvokePrintHello(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "printhello",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeDoubleSql(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "doubleSql",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeUnpredictableSql(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "unpredictableSql",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeCreatetable(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "createTable",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeCreatedb(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "createDb",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeCreateuesr(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "createUser",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeAuoIncrement(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[auoIncrement] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "autoIncrementSql",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func InvokeCommit(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_insert] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "commitSql",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func testGetTxByTxId(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, txId, chainId string) *commonPb.TransactionInfo {
	fmt.Println("========================================================================================================")
	fmt.Println("========================================================================================================")
	fmt.Println("========get tx by txId ", txId, "===============", time.Now().Format("2006-01-02 15:04:05"))
	fmt.Println("========================================================================================================")
	fmt.Println("========================================================================================================")
	fmt.Printf("\n============ get tx by txId [%s] ============\n", txId)

	// 构造Payload
	pair := &commonPb.KeyValuePair{Key: "txId", Value: []byte(txId)}
	var pairs []*commonPb.KeyValuePair
	pairs = append(pairs, pair)

	payloadBytes := constructPayload(syscontract.SystemContract_CHAIN_QUERY.String(), "GET_TX_BY_TX_ID", pairs)

	resp := proposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		chainId, txId, payloadBytes)

	result := &commonPb.TransactionInfo{}
	err := proto.Unmarshal(resp.ContractResult.Result, result)
	if err != nil {
		panic(err)
	}
	return result
}

func testWaitTx(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, txId string, count ...int) {
	if count != nil {
		if count[0] > 200 {
			panic("等待交易超时")
		}
	} else {
		count = []int{1}
	}
	time.Sleep(100 * time.Millisecond)
	fmt.Printf("\n============ testWaitTx [%s] ============%s\n", txId, time.Now().Format("2006-01-02 15:04:05"))
	// 构造Payload
	pair := &commonPb.KeyValuePair{Key: "txId", Value: []byte(txId)}
	var pairs []*commonPb.KeyValuePair
	pairs = append(pairs, pair)

	payloadBytes := constructPayload(syscontract.SystemContract_CHAIN_QUERY.String(), "GET_TX_BY_TX_ID", pairs)

	resp := proposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		chainId, txId, payloadBytes)
	if resp == nil || resp.ContractResult == nil || strings.Contains(resp.Message, "no such transaction") {
		testWaitTx(sk3, client, chainId, txId, count[0]+1)
	} else if resp != nil && len(resp.Message) != 0 {
		fmt.Println(resp.Message)
	}
}

func testInvokeSqlUpdate(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, id string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_update] [%s] ============\n", contractName, id)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(id),
		},
		{
			Key:   "name",
			Value: []byte("长安链chainmaker_update"),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_update",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func testInvokeSqlCommon(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, method string, chainId string, id string) (string, string) {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ common contract %s[%s] [%s] ============\n", contractName, method, id)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(id),
		},
		{
			Key:   "name",
			Value: []byte("长安链chainmaker_update"),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       method,
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId, string(resp.ContractResult.Result)
}
func testInvokeSqlUpdateRollbackDbSavePoint(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, id string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_update_rollback_save_point] [%s] ============\n", contractName, id)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(id),
		},
		{
			Key:   "name",
			Value: []byte("chainmaker_save_point"),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_update_rollback_save_point",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}
func testCrossCall(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[sql_cross_call] ============\n", contractName)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "contract_name",
			Value: []byte(contractName),
		},
		{
			Key:   "min_age",
			Value: []byte("4"),
		},
		{
			Key:   "max_age",
			Value: []byte("7"),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_cross_call",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func testInvokeSqlDelete(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, id string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ invoke contract %s[save] [%s] ============\n", contractName, txId)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(id),
		},
	}
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_delete",
		Parameters:   pairs,
	}

	resp := proposalRequest(sk3, client, commonPb.TxType_INVOKE_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	return txId
}

func testQuerySqlById(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string, id string) (string, string) {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ query contract %s[sql_query_by_id] id=%s ============\n", contractName, id)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "id",
			Value: []byte(id),
		},
	}

	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_query_by_id",
		Parameters:   pairs,
	}

	//payloadBytes, err := proto.Marshal(payload)
	//if err != nil {
	//	log.Fatalf(logTempMarshalPayLoadFailed, err.Error())
	//}

	resp := proposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		chainId, txId, payload)

	//fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	//fmt.Println(string(resp.ContractResult.Result))
	//items := serialize.EasyUnmarshal(resp.ContractResult.Result)
	//for _, item := range items {
	//	fmt.Println(item.Key, item.Value)
	//}
	return txId, string(resp.ContractResult.Result)
}

func testQuerySqlRangAge(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, chainId string) string {
	txId := utils.GetRandTxId()
	fmt.Printf("\n============ query contract %s[sql_query_range_of_age] ============\n", contractName)

	// 构造Payload
	pairs := []*commonPb.KeyValuePair{
		{
			Key:   "max_age",
			Value: []byte("4"),
		},
		{
			Key:   "min_age",
			Value: []byte("1"),
		},
	}

	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       "sql_query_range_of_age",
		Parameters:   pairs,
	}

	//payloadBytes, err := proto.Marshal(payload)
	//if err != nil {
	//	log.Fatalf(logTempMarshalPayLoadFailed, err.Error())
	//}

	resp := proposalRequest(sk3, client, commonPb.TxType_QUERY_CONTRACT,
		chainId, txId, payload)

	fmt.Printf(logTempSendTx, resp.Code, resp.Message, resp.ContractResult)
	fmt.Println(string(resp.ContractResult.Result))
	//items := serialize.EasyUnmarshal(resp.ContractResult.Result)
	//for _, item := range items {
	//	fmt.Println(item.Key, item.Value)
	//}
	return txId
}

func proposalRequest(sk3 crypto.PrivateKey, client *apiPb.RpcNodeClient, txType commonPb.TxType,
	chainId, txId string, payload *commonPb.Payload) *commonPb.TxResponse {
	payload.ChainId = chainId
	payload.TxType = txType
	payload.Timestamp = time.Now().Unix()
	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(10*time.Second))
	defer cancel()

	if txId == "" {
		txId = utils.GetRandTxId()

	}
	payload.TxId = txId

	file, err := ioutil.ReadFile(userCrtPath)
	if err != nil {
		panic(err)
	}

	// 构造Sender
	//pubKeyString, _ := sk3.PublicKey().String()
	sender := &acPb.Member{
		OrgId:      orgId,
		MemberInfo: file,
		////IsFullCert: true,
		//MemberInfo: []byte(pubKeyString),
	}

	// 构造Header
	//header := &commonPb.Payload{
	//	ChainId:        chainId,
	//	TxType:         txType,
	//	TxId:           txId,
	//	Timestamp:      time.Now().Unix(),
	//	ExpirationTime: 0,
	//}

	req := &commonPb.TxRequest{
		Payload: payload,
		Sender:  &commonPb.EndorsementEntry{Signer: sender},
	}

	// 拼接后，计算Hash，对hash计算签名
	rawTxBytes, err := utils.CalcUnsignedTxRequestBytes(req)
	if err != nil {
		log.Fatalf("CalcUnsignedTxRequest failed, %s", err.Error())
		os.Exit(1)
	}

	fmt.Errorf("################ %s", string(sender.MemberInfo))

	signer := getSigner(sk3, sender)
	signBytes, err := signer.Sign("SHA256", rawTxBytes)
	//signBytes, err := signer.Sign("SM3", rawTxBytes)
	if err != nil {
		log.Fatalf("sign failed, %s", err.Error())
		os.Exit(1)
	}

	req.Sender.Signature = signBytes

	result, err := (*client).SendRequest(ctx, req)

	if err != nil {
		statusErr, ok := status.FromError(err)
		if ok && statusErr.Code() == codes.DeadlineExceeded {
			fmt.Println("WARN: client.call err: deadline")
			os.Exit(1)
		}
		fmt.Printf("ERROR: client.call err: %v\n", err)
		os.Exit(1)
	}
	return result
}

func getSigner(sk3 crypto.PrivateKey, sender *acPb.Member) protocol.SigningMember {
	skPEM, err := sk3.String()
	if err != nil {
		log.Fatalf("get sk PEM failed, %s", err.Error())
	}
	//fmt.Printf("skPEM: %s\n", skPEM)

	signer, err := accesscontrol.NewCertSigningMember("", sender, skPEM, "")
	if err != nil {
		panic(err)
	}
	return signer
}

func initGRPCConnect(useTLS bool) (*grpc.ClientConn, error) {
	url := fmt.Sprintf("%s:%d", IP, Port)

	if useTLS {
		tlsClient := ca.CAClient{
			ServerName: "chainmaker.org",
			CaPaths:    caPaths,
			CertFile:   userCrtPath,
			KeyFile:    userKeyPath,
		}

		c, err := tlsClient.GetCredentialsByCA()
		if err != nil {
			log.Fatalf("GetTLSCredentialsByCA err: %v", err)
			return nil, err
		}
		return grpc.Dial(url, grpc.WithTransportCredentials(*c))
	} else {
		return grpc.Dial(url, grpc.WithInsecure())
	}
}
func initGRPCConnectOrg2(useTLS bool) (*grpc.ClientConn, error) {
	fmt.Println("============init org2 conn============")
	url := fmt.Sprintf("%s:%d", IPOrg2, PortOrg2)

	if useTLS {
		tlsClient := ca.CAClient{
			ServerName: "chainmaker.org",
			CaPaths:    caPaths,
			CertFile:   userCrtPathOrg2,
			KeyFile:    userKeyPathOrg2,
		}

		c, err := tlsClient.GetCredentialsByCA()
		if err != nil {
			log.Fatalf("GetTLSCredentialsByCA err: %v", err)
			return nil, err
		}
		return grpc.Dial(url, grpc.WithTransportCredentials(*c))
	} else {
		return grpc.Dial(url, grpc.WithInsecure())
	}
}

func constructPayload(contractName, method string, pairs []*commonPb.KeyValuePair) *commonPb.Payload {
	payload := &commonPb.Payload{
		ContractName: contractName,
		Method:       method,
		Parameters:   pairs,
	}

	return payload
}

//	func acSign(msg *commonPb.Payload, orgIdList []int) ([]*commonPb.EndorsementEntry, error) {
//		msg.Endorsement = nil
//		bytes, _ := proto.Marshal(msg)
//
//		signers := make([]protocol.SigningMember, 0)
//		for _, orgId := range orgIdList {
//
//			numStr := strconv.Itoa(orgId)
//			path := fmt.Sprintf(prePathFmt, numStr) + "admin1.sign.key"
//			file, err := ioutil.ReadFile(path)
//			if err != nil {
//				panic(err)
//			}
//			sk, err := asym.PrivateKeyFromPEM(file, nil)
//			if err != nil {
//				panic(err)
//			}
//
//			userCrtPath := fmt.Sprintf(prePathFmt, numStr) + "admin1.sign.crt"
//			file2, err := ioutil.ReadFile(userCrtPath)
//			//fmt.Println("node", orgId, "crt", string(file2))
//			if err != nil {
//				panic(err)
//			}
//
//			// 获取peerId
//			_, err = helper.GetLibp2pPeerIdFromCert(file2)
//			//fmt.Println("node", orgId, "peerId", peerId)
//
//			// 构造Sender
//			sender1 := &acPb.Member{
//				OrgId:      "yz-org" + numStr + ".chainmaker.org",
//				MemberInfo: file2,
//				//IsFullCert: true,
//			}
//
//			signer := getSigner(sk, sender1)
//			signers = append(signers, signer)
//		}
//
//		return accesscontrol.MockSignWithMultipleNodes(bytes, signers, "SHA256")
//	}
func panicNotEqual(a string, b string) {
	if a != b {
		panic(a + " not equal " + b)
	}
}
