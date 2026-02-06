/*
 * Copyright 2020 The SealEVM Authors
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

package precompiledContracts

import (
	"math/big"

	"chainmaker.org/chainmaker/common/v2/crypto/asym/sm2"

	"chainmaker.org/chainmaker/common/v2/crypto"

	"chainmaker.org/chainmaker/common/v2/crypto/asym"
	"chainmaker.org/chainmaker/vm-evm/v2/evm-go/params"
	"chainmaker.org/chainmaker/vm-evm/v2/evm-go/utils"
)

var (
	falseBytes = make([]byte, 32)
	trueBytes  = []byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}
	sm2Opt     = crypto.SignOpts{Hash: crypto.HASH_TYPE_SM3, UID: crypto.CRYPTO_DEFAULT_UID}
)

// signVerify enables users to log in the contract
type signVerify struct{}

//func (s *signVerify)SetValue(v string){}

// RequiredGas returns the gas required to execute the pre-compiled contract.
func (s *signVerify) GasCost(input []byte) uint64 {
	return params.EcrecoverGas
}

func (s *signVerify) Execute(input []byte) ([]byte, error) {
	return verify(input)
}

func readBytes(input *[]byte, size uint64) (data []byte, err error) {
	if uint64(len(*input)) < size || size == 0 {
		return nil, utils.ErrVerifyInput
	}

	data = (*input)[:size]
	*input = (*input)[size:]
	return data, nil
}

func readUint64(input *[]byte) (uint64, error) {
	const evmWordLen = 32
	if uint64(len(*input)) < evmWordLen {
		return 0, utils.ErrVerifyInput
	}

	data := (*input)[:evmWordLen]
	*input = (*input)[evmWordLen:]
	return new(big.Int).SetBytes(data).Uint64(), nil
}

// verify implements the signVerify precompile
// input |--public key len--|--public key data--|--msg len--|--msg data--|--sign len--|--sign data--| */
//
//	32 bytes                               32 bytes                 32 bytes
func verify(input []byte) ([]byte, error) {
	//get serialized public key length
	pkLen, err := readUint64(&input)
	if err != nil {
		return falseBytes, err
	}

	//get serialized public key
	pkBytes, err1 := readBytes(&input, pkLen)
	if err1 != nil {
		return falseBytes, err1
	}

	//pubKey, err := smx509.ParseSm2PublicKey(pkBytes)
	pubKey, err2 := asym.PublicKeyFromDER(pkBytes)
	if err2 != nil {
		pubKey, err2 = asym.PublicKeyFromPEM(pkBytes)
		if err2 != nil {
			k, e := asym.ParseSM2PublicKey(pkBytes)
			if e != nil {
				return falseBytes, e
			}

			pubKey = &sm2.PublicKey{K: k}
		}
	}

	//get message length
	msgLen, err3 := readUint64(&input)
	if err3 != nil {
		return falseBytes, err3
	}

	//get message
	msg, err4 := readBytes(&input, msgLen)
	if err4 != nil {
		return falseBytes, err4
	}

	//get signature length
	signLen, err5 := readUint64(&input)
	if err5 != nil {
		return falseBytes, err5
	}

	//get signature
	sign, err6 := readBytes(&input, signLen)
	if err6 != nil {
		return falseBytes, err6
	}

	//opt := &crypto.SignOpts{Hash: crypto.HASH_TYPE_SM3, UID: crypto.CRYPTO_DEFAULT_UID}
	ret, err7 := pubKey.VerifyWithOpts(msg, sign, &sm2Opt)
	if err7 != nil || ret != true {
		return falseBytes, utils.ErrVerifyInput
	}

	return trueBytes, nil
}
