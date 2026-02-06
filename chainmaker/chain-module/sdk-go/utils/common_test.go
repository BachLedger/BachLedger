/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package utils

import (
	"encoding/hex"
	"fmt"
	"testing"
	"time"

	"chainmaker.org/chainmaker/pb-go/v2/common"
	"github.com/stretchr/testify/require"
)

func TestGetRandTxId(t *testing.T) {
	txId := GetRandTxId()
	require.Len(t, txId, 64)
}

func TestCheckProposalRequestResp(t *testing.T) {
	tests := []struct {
		name               string
		serverTxResp       *common.TxResponse
		needContractResult bool
		wantErr            bool
	}{
		{
			"good",
			&common.TxResponse{Code: common.TxStatusCode_SUCCESS, ContractResult: &common.ContractResult{
				Code: SUCCESS,
			}},
			true,
			false,
		},
		{
			"bad",
			&common.TxResponse{Code: common.TxStatusCode_CONTRACT_FAIL},
			false,
			true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := CheckProposalRequestResp(tt.serverTxResp, tt.needContractResult)
			require.Equal(t, err != nil, tt.wantErr)
		})
	}
}

func TestGetTimestampTxId(t *testing.T) {
	tests := []struct {
		name string
	}{
		{
			name: "正常流",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := GetTimestampTxId()
			t.Log(got)
			require.Len(t, got, 64)
		})
	}
}

func TestGetNanosecondByTxId(t *testing.T) {
	nano := time.Now().UnixNano()
	type args struct {
		nano int64
	}
	tests := []struct {
		name string
		args args
	}{
		{
			name: "正常流",
			args: args{nano: nano},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := GetTimestampTxIdByNano(tt.args.nano)
			nanosecond, err := GetNanoByTimestampTxId(got)
			if err != nil {
				return
			}

			require.Truef(t, nanosecond == nano, "emmm not ok")
		})
	}
}

func TestHexEB(t *testing.T) {
	b, err := hex.DecodeString("ca")
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println(b)
}

func TestGetCertificateId(t *testing.T) {
	crt := []byte(`-----BEGIN CERTIFICATE-----
MIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ
MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt
b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD
ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1
MTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw
DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn
MQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j
aGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax
9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY
KYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG
A1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME
JDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD
AgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm
Aod1WZHJajgnDQ2zEcFF94aejR9dmGBB/P//
-----END CERTIFICATE-----`)
	h, err := GetCertificateId(crt, "SHA256")
	require.Nil(t, err)
	require.Equal(t, "adc50a433a15dd1ab43fdbc47f169f36b3207d53e0f1f05e22c23e1709a03a78", hex.EncodeToString(h))
}

func TestParseCert(t *testing.T) {
	crt := []byte(`-----BEGIN CERTIFICATE-----
MIIChzCCAi2gAwIBAgIDAwGbMAoGCCqGSM49BAMCMIGKMQswCQYDVQQGEwJDTjEQ
MA4GA1UECBMHQmVpamluZzEQMA4GA1UEBxMHQmVpamluZzEfMB0GA1UEChMWd3gt
b3JnMS5jaGFpbm1ha2VyLm9yZzESMBAGA1UECxMJcm9vdC1jZXJ0MSIwIAYDVQQD
ExljYS53eC1vcmcxLmNoYWlubWFrZXIub3JnMB4XDTIwMTIwODA2NTM0M1oXDTI1
MTIwNzA2NTM0M1owgY8xCzAJBgNVBAYTAkNOMRAwDgYDVQQIEwdCZWlqaW5nMRAw
DgYDVQQHEwdCZWlqaW5nMR8wHQYDVQQKExZ3eC1vcmcxLmNoYWlubWFrZXIub3Jn
MQ4wDAYDVQQLEwVhZG1pbjErMCkGA1UEAxMiYWRtaW4xLnNpZ24ud3gtb3JnMS5j
aGFpbm1ha2VyLm9yZzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABORqoYNAw8ax
9QOD94VaXq1dCHguarSKqAruEI39dRkm8Vu2gSHkeWlxzvSsVVqoN6ATObi2ZohY
KYab2s+/QA2jezB5MA4GA1UdDwEB/wQEAwIBpjAPBgNVHSUECDAGBgRVHSUAMCkG
A1UdDgQiBCDZOtAtHzfoZd/OQ2Jx5mIMgkqkMkH4SDvAt03yOrRnBzArBgNVHSME
JDAigCA1JD9xHLm3xDUukx9wxXMx+XQJwtng+9/sHFBf2xCJZzAKBggqhkjOPQQD
AgNIADBFAiEAiGjIB8Wb8mhI+ma4F3kCW/5QM6tlxiKIB5zTcO5E890CIBxWDICm
Aod1WZHJajgnDQ2zEcFF94aejR9dmGBB/P//
-----END CERTIFICATE-----`)
	_, err := ParseCert(crt)
	require.Nil(t, err)
}

func TestExists(t *testing.T) {
	exists := Exists("/sHFBf2xCJZzAKBggqhkjOPQQD")
	require.False(t, exists)
}

func TestIsArchived(t *testing.T) {
	isArchived := IsArchived(common.TxStatusCode_ARCHIVED_BLOCK)
	require.True(t, isArchived)
}

func TestIsArchivedString(t *testing.T) {
	isArchived := IsArchivedString(common.TxStatusCode_ARCHIVED_BLOCK.String())
	require.True(t, isArchived)
}
