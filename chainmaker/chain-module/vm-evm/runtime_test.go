/*
 * Copyright (C) BABEC. All rights reserved.
 * Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package evm

import (
	"crypto/elliptic"
	"encoding/hex"
	"fmt"
	"io/ioutil"
	"os"
	"strings"
	"testing"

	"github.com/tjfoc/gmsm/x509"

	"chainmaker.org/chainmaker/common/v2/crypto"
	"chainmaker.org/chainmaker/common/v2/crypto/asym/sm2"

	"chainmaker.org/chainmaker/common/v2/evmutils/abi"

	bcx509 "chainmaker.org/chainmaker/common/v2/crypto/x509"
	"chainmaker.org/chainmaker/utils/v2"

	"chainmaker.org/chainmaker/logger/v2"
	pbac "chainmaker.org/chainmaker/pb-go/v2/accesscontrol"
	"chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/protocol/v2"
	"chainmaker.org/chainmaker/vm-evm/v2/test"
)

const (
	chainId      = "chain01"
	OrgId1       = "yz-org1.yzchain.org"
	certFilePath = "./test/config/admin1.sing.crt"
	txId         = "TX_ID_XXX"

	contractVersion = "v1.0.0"
	initMethod      = "init_contract"
	tokenName       = "contract_token"
	tokenBinPath    = "./test/contracts/contract01/token.bin"
	tokenBodyFile   = "./test/contracts/contract01/token_body.bin"

	contractCName = "contract_C"
	contractCBin  = "./test/contracts/contract03/C.bin"
	contractCBody = "./test/contracts/contract03/C_body.bin"
	contractCAbi  = "./test/contracts/contract03/C.abi"
)

var sm2Opt = crypto.SignOpts{Hash: crypto.HASH_TYPE_SM3, UID: crypto.CRYPTO_DEFAULT_UID}

func TestInstallTemplate(t *testing.T) {
	//installTemplate(t, "./test/contracts/contract02/", "Bar")
	installTemplate(t, "./test/contracts/contract02/", "sm2Verify")

}

func TestInvokeTemplate(t *testing.T) {
	//InvokeTemplate(t, "./test/contracts/contract02/", "Bar", "tryCatchNewContract", 0x0000000000000000000000000000000000000000)

	/************* style 1: test sm2 verify precompile contract **********/
	pkBytes, msg, sig := prepareSm2VerifyTest(1)
	InvokeTemplate(t, "./test/contracts/contract02/", "sm2Verify", "verify", pkBytes, msg, sig)

	/************* style 2: test sm2 verify precompile contract **********/
	pkBytes, msg, sig = prepareSm2VerifyTest(2)
	InvokeTemplate(t, "./test/contracts/contract02/", "sm2Verify", "verify", pkBytes, msg, sig)

	/************* style 3: test sm2 verify precompile contract **********/
	pkBytes, msg, sig = prepareSm2VerifyTest(3)
	InvokeTemplate(t, "./test/contracts/contract02/", "sm2Verify", "verify", pkBytes, msg, sig)

	/************* style 4: test sm2 verify precompile contract **********/
	pkBytes, msg, sig = prepareSm2VerifyTest(4)
	InvokeTemplate(t, "./test/contracts/contract02/", "sm2Verify", "verify", pkBytes, msg, sig)
}

func installTemplate(t *testing.T, path, name string, args ...interface{}) {
	//部署合约
	method := initMethod
	test.ContractName = name
	test.CertFilePath = certFilePath
	test.ByteCodeFile = path + name + ".bin"
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)
	//调用合约
	abiJson, err := ioutil.ReadFile(path + name + ".abi")
	if err != nil {
		loggerByChain.Errorf("Read ABI file failed, err:%v", err.Error())
	}

	myAbi, err := abi.JSON(strings.NewReader(string(abiJson)))
	if err != nil {
		loggerByChain.Errorf("constrcut ABI obj failed, err:%v", err.Error())
	}

	dataByte, err := myAbi.Pack("", args)
	if err != nil {
		loggerByChain.Errorf("create ABI data failed, err:%v", err.Error())
	}

	dataString := hex.EncodeToString(dataByte)
	byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	parameters[protocol.ContractEvmParamKey] = []byte(dataString)
	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("ContractResult Result:%+X", contractResult.Result)

	fd, _ := os.OpenFile(path+name+".body", os.O_RDWR|os.O_CREATE, 0766)
	fd.Write(contractResult.Result)
	fd.Close()
}

func InvokeTemplate(t *testing.T, path, name, method string, args ...interface{}) {
	test.ContractName = name
	test.ByteCodeFile = path + name + ".body"
	test.CertFilePath = certFilePath
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)

	//调用合约
	abiJson, err := ioutil.ReadFile(path + name + ".abi")
	if err != nil {
		loggerByChain.Errorf("Read ABI file failed, err:%v", err.Error())
	}

	myAbi, err := abi.JSON(strings.NewReader(string(abiJson)))
	if err != nil {
		loggerByChain.Errorf("constrcut ABI obj failed, err:%v", err.Error())
	}

	dataByte, err := myAbi.Pack(method, args...)
	if err != nil {
		loggerByChain.Errorf("create ABI data failed, err:%v", err.Error())
	}

	dataString := hex.EncodeToString(dataByte)
	method = dataString[0:8]

	//byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	parameters[protocol.ContractEvmParamKey] = []byte(dataString)

	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("method store-- ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("method store-- ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("method store-- ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("method store-- ContractResult Result:%+X", contractResult.Result)
}

func TestInstallContractToken(t *testing.T) {
	//部署合约
	method := initMethod
	test.CertFilePath = certFilePath
	test.ContractName = tokenName
	test.ByteCodeFile = tokenBinPath
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)

	byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	parameters["data"] = []byte("00000000000000000000000013f0c1639a9931b0ce17e14c83f96d4732865b58")
	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("ContractResult Result:%+X", contractResult.Result)
}

func TestInvokeToken(t *testing.T) {
	//调用合约
	method := "4f9d719e" //method testEvent
	test.ContractName = tokenName
	test.ByteCodeFile = tokenBodyFile
	test.CertFilePath = certFilePath
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)

	byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	parameters["data"] = []byte("4f9d719e")

	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("method testEvent-- ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("method testEvent-- ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("method testEvent-- ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("method testEvent-- ContractResult Result:%+X", contractResult.Result)
}

func TestInstallC(t *testing.T) {
	//部署合约
	method := initMethod
	test.ContractName = contractCName
	test.CertFilePath = certFilePath
	test.ByteCodeFile = contractCBin
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)

	byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	//parameters["data"] = []byte("00000000000000000000000013f0c1639a9931b0ce17e14c83f96d4732865b58")
	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("ContractResult Result:%+X", contractResult.Result)
}

func TestInvokeC(t *testing.T) {
	test.ContractName = contractCName
	test.ByteCodeFile = contractCBody
	test.CertFilePath = certFilePath
	parameters := make(map[string][]byte)
	contractId, txContext, byteCode := test.InitContextTest(common.RuntimeType_EVM, t)

	runtimeInstance := &RuntimeInstance{
		ChainId:      chainId,
		Log:          logger.GetLogger(logger.MODULE_VM),
		TxSimContext: txContext,
	}

	loggerByChain := logger.GetLoggerByChain(logger.MODULE_VM, chainId)

	//调用合约
	abiJson, err := ioutil.ReadFile(contractCAbi)
	if err != nil {
		loggerByChain.Errorf("Read C ABI file failed, err:%v", err.Error())
	}

	myAbi, err := abi.JSON(strings.NewReader(string(abiJson)))
	if err != nil {
		loggerByChain.Errorf("constrcut ABI obj failed, err:%v", err.Error())
	}

	dataByte, err := myAbi.Pack("createDSalted", 5, "contract_D")
	if err != nil {
		loggerByChain.Errorf("create ABI data failed, err:%v", err.Error())
	}

	dataString := hex.EncodeToString(dataByte)
	method := dataString[0:8]
	//method2 := hex.EncodeToString(evmutils.Keccak256([]byte("createDSalted(uint256,string)")))[0:8]
	//loggerByChain.Infof("method:%v, method2:%v", method, method2)

	//method := "a339d707"
	//dataString := "a339d70700000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a636f6e74726163745f4400000000000000000000000000000000000000000000"

	byteCode, _ = hex.DecodeString(string(byteCode))
	test.BaseParam(parameters)
	parameters[protocol.ContractCreatorPkParam] = contractId.Creator.MemberInfo
	parameters[protocol.ContractSenderPkParam] = txContext.GetSender().MemberInfo
	parameters[protocol.ContractEvmParamKey] = []byte(dataString)

	contractResult, _ := runtimeInstance.Invoke(contractId, method, byteCode, parameters, txContext, 0)
	loggerByChain.Infof("method store-- ContractResult Code:%+v", contractResult.Code)
	loggerByChain.Infof("method store-- ContractResult ContractEvent:%+v", contractResult.ContractEvent)
	loggerByChain.Infof("method store-- ContractResult Message:%+v", contractResult.Message)
	loggerByChain.Infof("method store-- ContractResult Result:%+X", contractResult.Result)
}

//func TestConvertEvmContractName(t *testing.T) {
//	name := "0x7162629f540a9e19eCBeEa163eB8e48eC898Ad00"
//	addr, _ := contractNameToAddress(name)
//	t.Logf("evm addr:%s", addr.Text(16))
//	assert.Equal(t, strings.ToLower(name[2:]), addr.Text(16))
//}

func mockSender() *pbac.Member {
	file, err := ioutil.ReadFile(certFilePath)
	if err != nil {
		panic("file is nil" + err.Error())
	}

	return &pbac.Member{
		OrgId:      OrgId1,
		MemberType: pbac.MemberType_CERT,
		MemberInfo: file,
	}
}

func mockContract(name string, cert []byte) *common.Contract {
	addr, _ := utils.NameToAddrStr(name, configPb.AddrType_ETHEREUM, 2300)

	return &common.Contract{
		Name:        name,
		Version:     contractVersion,
		RuntimeType: common.RuntimeType_EVM,
		Status:      common.ContractStatus_NORMAL,
		Creator: &pbac.MemberFull{
			OrgId:      OrgId1,
			MemberType: pbac.MemberType_CERT,
			MemberInfo: cert,
		},
		Address: addr,
	}
}

func mockTx() *common.Transaction {
	return &common.Transaction{
		Payload: &common.Payload{
			ChainId:        chainId,
			TxType:         common.TxType_INVOKE_CONTRACT,
			TxId:           txId,
			Timestamp:      0,
			ExpirationTime: 0,
		},
		Result: nil,
	}
}

func mockParams(cert *bcx509.Certificate) map[string][]byte {
	parameters := make(map[string][]byte)

	parameters[protocol.ContractTxIdParam] = []byte(txId)
	parameters[protocol.ContractCreatorOrgIdParam] = []byte(OrgId1)
	parameters[protocol.ContractCreatorRoleParam] = []byte("admin")
	parameters[protocol.ContractCreatorPkParam] = []byte(hex.EncodeToString(cert.SubjectKeyId))
	parameters[protocol.ContractSenderOrgIdParam] = []byte(OrgId1)
	parameters[protocol.ContractSenderRoleParam] = []byte("user")
	parameters[protocol.ContractSenderPkParam] = []byte(hex.EncodeToString(cert.SubjectKeyId))
	parameters[protocol.ContractBlockHeightParam] = []byte("1")

	return parameters
}

func marshalSm2PublicKey(publicKey crypto.PublicKey, marshalMode int) []byte {
	var pkBytes []byte

	switch marshalMode {
	case 1:
		pkBytes, _ = publicKey.Bytes()
	case 2:
		pkStr, _ := publicKey.String()
		pkBytes = []byte(pkStr)
	case 3:
		k := publicKey.(*sm2.PublicKey).K
		pkBytes, _ = x509.MarshalSm2PublicKey(k)
	case 4:
		k := publicKey.(*sm2.PublicKey).K
		pkBytes = elliptic.Marshal(k.Curve, k.X, k.Y)
	}

	return pkBytes
}

func prepareSm2VerifyTest(marshalMode int) (pubKey, message, signature []byte) {
	msg := []byte("test sm2 verify")

	/****************** generate key *****************/
	priKey, _ := sm2.New(crypto.SM2)
	sig, _ := priKey.SignWithOpts(msg, &sm2Opt)

	/******* ensure that the signature is approved *************/
	pk := priKey.PublicKey()
	ret, err := pk.VerifyWithOpts(msg, sig, &sm2Opt)
	if err != nil {
		fmt.Printf("verify failed:%v", ret)
		panic("verify failed.")
	}

	/******** serialize public key ******************/
	pkBytes := marshalSm2PublicKey(pk, marshalMode)

	return pkBytes, msg, sig
}
