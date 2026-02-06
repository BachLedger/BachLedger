/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	dpospb "chainmaker.org/chainmaker/pb-go/v2/consensus/dpos"
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"math"
	"sort"
	"strconv"
	"strings"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	configPb "chainmaker.org/chainmaker/pb-go/v2/config"
	"chainmaker.org/chainmaker/pb-go/v2/syscontract"

	"github.com/gogo/protobuf/proto"
	"github.com/mr-tron/base58/base58"
)

const (
	// DPosOrgId DPoS org id
	DPosOrgId = "dpos_org_id"

	defaultDPoSMinSelfDelegation            = "250000000000000000000000"
	defaultDPoSEpochBlockNumber             = 1000
	defaultDPoSEpochValidatorNumber         = 4
	defaultDPoSCompletionUnboundingEpochNum = 1
)

const (
	keyCurrentEpoch         = "CE"
	keyMinSelfDelegation    = "MSD"
	keyEpochFormat          = "E/%s"
	keyDelegationFormat     = "D/%s/%s"
	keyValidatorFormat      = "V/%s"
	keyEpochValidatorNumber = "EVN"
	keyEpochBlockNumber     = "EBN"
	keyUnbondingEpochNumber = "UEN"
	keyNodeIDFormat         = "N/%s"
	keyRevNodeFormat        = "NR/%s"

	keyERC20Total             = "erc20.total"
	keyERC20Owner             = "erc20.owner"
	keyERC20Decimals          = "erc20.decimals"
	keyERC20Acc               = "erc20.account:"
	keyStakeMinSelfDelegation = "stake.minSelfDelegation"
	keyStakeEpochValidatorNum = "stake.epochValidatorNum"
	keyStakeEpochBlockNum     = "stake.epochBlockNum"
	keyStakeUnbondingEpochNum = "stake.completionUnbondingEpochNum"
	keyStakeCandidate         = "stake.candidate"
	keyStakeConfigNodeID      = "stake.nodeID"
)

// ERC20Config for DPoS
type ERC20Config struct {
	total    *BigInteger
	owner    string
	decimals *BigInteger
	accounts []*struct {
		address string
		token   *BigInteger
	}
}

func newERC20Config() *ERC20Config {
	return &ERC20Config{
		accounts: make([]*struct {
			address string
			token   *BigInteger
		}, 0),
	}
}

func (e *ERC20Config) addAccount(address string, token *BigInteger) error {
	// 需要判断是否有重复，每个地址只允许配置一次token
	for i := 0; i < len(e.accounts); i++ {
		if e.accounts[i].address == address {
			return fmt.Errorf("token of address[%s] cannot be set more than once", address)
		}
	}
	e.accounts = append(e.accounts, &struct {
		address string
		token   *BigInteger
	}{address: address, token: token})
	return nil
}

// toTxWrites convert to TxWrites
func (e *ERC20Config) toTxWrites() []*commonPb.TxWrite {
	contractName := syscontract.SystemContract_DPOS_ERC20.String()
	txWrites := []*commonPb.TxWrite{
		{
			Key:          []byte("OWN"), // equal with native.KeyOwner
			Value:        []byte(e.owner),
			ContractName: contractName,
		},
		{
			Key:          []byte("DEC"), // equal with native.KeyDecimals
			Value:        []byte(e.decimals.String()),
			ContractName: contractName,
		},
		{
			Key:          []byte("TS"), // equal with native.KeyTotalSupply
			Value:        []byte(e.total.String()),
			ContractName: contractName,
		},
	}

	// 添加accounts的读写集
	sort.SliceStable(e.accounts, func(i, j int) bool {
		return e.accounts[i].address < e.accounts[j].address
	})
	for i := 0; i < len(e.accounts); i++ {
		txWrites = append(txWrites, &commonPb.TxWrite{
			Key:          []byte(fmt.Sprintf("B/%s", e.accounts[i].address)),
			Value:        []byte(e.accounts[i].token.String()),
			ContractName: contractName,
		})
	}
	return txWrites
}

func (e *ERC20Config) loadToken(address string) *BigInteger {
	for i := 0; i < len(e.accounts); i++ {
		if e.accounts[i].address == address {
			return e.accounts[i].token
		}
	}
	return nil
}

func (e *ERC20Config) legal() error {
	if len(e.accounts) == 0 {
		return fmt.Errorf("account's size must more than zero")
	}
	// 其他信息已校验过，当前只需要校验所有账户的token和为total即可
	sum := NewZeroBigInteger()
	for i := 0; i < len(e.accounts); i++ {
		sum.Add(e.accounts[i].token)
	}
	// 比较sum与total
	if sum.Cmp(e.total) != 0 {
		return fmt.Errorf("sum of token is not equal with total, sum[%s] total[%s]", sum.String(), e.total.String())
	}
	return nil
}

// loadERC20Config load config of erc20 contract

// StakeConfig defines stake config
type StakeConfig struct {
	minSelfDelegation string
	validatorNum      uint64
	eachEpochNum      uint64
	unbondingEpochNum uint64
	candidates        []*dpospb.CandidateInfo
	nodeIDs           map[string]string // userAddr => nodeID
}

func (s *StakeConfig) toTxWrites() ([]*commonPb.TxWrite, error) {
	var (
		valNum                = make([]byte, 8)
		epochNum              = make([]byte, 8)
		completeUnboundingNum = make([]byte, 8)
	)
	binary.BigEndian.PutUint64(valNum, s.validatorNum)
	binary.BigEndian.PutUint64(epochNum, s.eachEpochNum)
	binary.BigEndian.PutUint64(completeUnboundingNum, s.unbondingEpochNum)

	// 1. add property in rwSets
	rwSets := []*commonPb.TxWrite{
		{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(keyMinSelfDelegation),
			Value:        []byte(s.minSelfDelegation),
		},
		{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(keyEpochValidatorNumber),
			Value:        valNum,
		},
		{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(keyEpochBlockNumber),
			Value:        epochNum,
		},
		{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(keyUnbondingEpochNumber),
			Value:        completeUnboundingNum,
		},
	}

	// 2. add validatorInfo, delegationInfo in rwSet
	sort.SliceStable(s.candidates, func(i, j int) bool {
		return s.candidates[i].PeerId < s.candidates[j].PeerId
	})
	validators := make([][]byte, 0, len(s.candidates))
	delegations := make([][]byte, 0, len(s.candidates))
	for _, candidate := range s.candidates {
		bz, err := proto.Marshal(&syscontract.Validator{
			Jailed:                     false,
			Status:                     syscontract.BondStatus_BONDED,
			Tokens:                     candidate.Weight,
			ValidatorAddress:           candidate.PeerId,
			DelegatorShares:            candidate.Weight,
			SelfDelegation:             candidate.Weight,
			UnbondingEpochId:           math.MaxInt64,
			UnbondingCompletionEpochId: math.MaxUint64,
		})
		if err != nil {
			return nil, err
		}
		validators = append(validators, bz)

		delegateBz, err := proto.Marshal(&syscontract.Delegation{
			DelegatorAddress: candidate.PeerId,
			ValidatorAddress: candidate.PeerId,
			Shares:           candidate.Weight,
		})
		if err != nil {
			return nil, err
		}
		delegations = append(delegations, delegateBz)
	}
	for i, validator := range s.candidates {
		rwSets = append(rwSets, &commonPb.TxWrite{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(fmt.Sprintf(keyValidatorFormat, validator.PeerId)),
			Value:        validators[i],
		})
		// key: prefix|delegator|validator
		key := []byte(fmt.Sprintf(keyDelegationFormat, validator.PeerId, validator.PeerId))
		rwSets = append(rwSets, &commonPb.TxWrite{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          key,
			Value:        delegations[i], // val: delegation info
		})
	}

	// 4. add epoch info
	valAddrs := make([]string, 0, len(s.candidates))
	for _, v := range s.candidates {
		valAddrs = append(valAddrs, v.PeerId)
	}
	epochInfo, err := proto.Marshal(&syscontract.Epoch{
		EpochId:               0,
		ProposerVector:        valAddrs,
		NextEpochCreateHeight: s.eachEpochNum,
	})
	if err != nil {
		return nil, err
	}
	rwSets = append(rwSets, &commonPb.TxWrite{
		ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
		Key:          []byte(keyCurrentEpoch), // key: prefix
		Value:        epochInfo,               // val: epochInfo
	})
	rwSets = append(rwSets, &commonPb.TxWrite{
		ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
		Key:          []byte(fmt.Sprintf(keyEpochFormat, "0")), // key: prefix|epochID
		Value:        epochInfo,                                // val: epochInfo
	})

	for _, addr := range valAddrs {
		rwSets = append(rwSets, &commonPb.TxWrite{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(fmt.Sprintf(keyNodeIDFormat, addr)), // key: prefix|addr
			Value:        []byte(s.nodeIDs[addr]),                    // val: nodeID
		})
		rwSets = append(rwSets, &commonPb.TxWrite{
			ContractName: syscontract.SystemContract_DPOS_STAKE.String(),
			Key:          []byte(fmt.Sprintf(keyRevNodeFormat, s.nodeIDs[addr])), // key: prefix|nodeID
			Value:        []byte(addr),                                           // val: addr
		})
	}
	return rwSets, nil
}

// getContractAddress 返回质押合约地址
func getContractAddress() string {
	bz := sha256.Sum256([]byte(syscontract.SystemContract_DPOS_STAKE.String()))
	return base58.Encode(bz[:])
}

// getSumToken 返回所有token的值
func (s *StakeConfig) getSumToken() *BigInteger {
	sum := NewZeroBigInteger()
	for i := 0; i < len(s.candidates); i++ {
		sum.Add(NewBigInteger(s.candidates[i].Weight))
	}
	return sum
}

func (s *StakeConfig) setCandidate(key, value string) error {
	values := strings.Split(key, ":")
	if len(values) != 2 {
		return fmt.Errorf("stake.candidate config error, actual: %s, expect: %s:<addr1>", key, keyStakeCandidate)
	}
	if err := isValidBigInt(value); err != nil {
		return fmt.Errorf("stake.candidate amount error, reason: %s", err)
	}
	s.candidates = append(s.candidates, &dpospb.CandidateInfo{
		PeerId: values[1], Weight: value,
	})
	return nil
}

func (s *StakeConfig) setNodeID(key, value string) error {
	values := strings.Split(key, ":")
	if len(values) != 2 {
		return fmt.Errorf("stake.nodeIDs config error, actual: %s, expect: %s:<addr1>", key, keyStakeConfigNodeID)
	}
	s.nodeIDs[values[1]] = value
	return nil
}

func (s *StakeConfig) isLegal() error {
	if len(s.nodeIDs) != len(s.candidates) {
		return fmt.Errorf("config nodeIDs and candidates not matched, nodeIDs num: %d, candidates: %d ",
			len(s.nodeIDs), len(s.candidates))
	}
	if len(s.candidates) != int(s.validatorNum) {
		return fmt.Errorf("The number of candidates[%d] is not equal "+
			"the required number of validator[%d] ", len(s.candidates), s.validatorNum)
	}
	return nil
}

func loadStakeConfig(consensusExtConfig []*configPb.ConfigKeyValue) (*StakeConfig, error) {
	/**
	  stake合约的配置
	  ext_config: # 扩展字段，记录难度、奖励等其他类共识算法配置
	    - key: stake.minSelfDelegation
	      value: 1000000000000
	    - key: stake.epochValidatorNum
	      value: 10
	    - key: stake.epochBlockNum
	      value: 2000
	    - key: stake.completionUnbondingEpochNum
	      value: 1
		- key: stake.candidate:<addr1>
	      value: 800000
		- key: stake.candidate:<addr2>
	      value: 600000
		- key: stake.nodeID:<addr1>
		  value: nodeID
	*/
	config := &StakeConfig{
		nodeIDs: make(map[string]string),
	}
	for _, kv := range consensusExtConfig {
		switch kv.Key {
		case keyStakeEpochBlockNum:
			val, err := strconv.ParseUint(kv.Value, 10, 64)
			if err != nil {
				return nil, err
			}
			config.eachEpochNum = val
		case keyStakeEpochValidatorNum:
			val, err := strconv.ParseUint(kv.Value, 10, 64)
			if err != nil {
				return nil, err
			}
			config.validatorNum = val
		case keyStakeMinSelfDelegation:
			if err := isValidBigInt(kv.Value); err != nil {
				return nil, fmt.Errorf("%s error, reason: %s", keyStakeMinSelfDelegation, err)
			}
			config.minSelfDelegation = kv.Value
		case keyStakeUnbondingEpochNum:
			val, err := strconv.ParseUint(kv.Value, 10, 64)
			if err != nil {
				return nil, err
			}
			config.unbondingEpochNum = val
		default:
			if strings.HasPrefix(kv.Key, keyStakeCandidate) {
				if err := config.setCandidate(kv.Key, kv.Value); err != nil {
					return nil, err
				}
			}
			if strings.HasPrefix(kv.Key, keyStakeConfigNodeID) {
				if err := config.setNodeID(kv.Key, kv.Value); err != nil {
					return nil, err
				}
			}
		}
	}
	if len(config.minSelfDelegation) == 0 {
		config.minSelfDelegation = defaultDPoSMinSelfDelegation
	}
	if config.eachEpochNum == 0 {
		config.eachEpochNum = defaultDPoSEpochBlockNumber
	}
	if config.unbondingEpochNum == 0 {
		config.unbondingEpochNum = defaultDPoSCompletionUnboundingEpochNum
	}
	if config.validatorNum == 0 {
		config.validatorNum = defaultDPoSEpochValidatorNumber
	}
	return config, nil
}
