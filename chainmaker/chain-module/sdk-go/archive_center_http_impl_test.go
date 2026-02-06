package chainmaker_sdk_go

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"reflect"
	"testing"

	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/sdk-go/v2/utils"
	"github.com/stretchr/testify/assert"
)

// Response 通用http返回信息
type Response struct {
	Code     int         `json:"code,omitempty"` // 错误码,0代表成功.其余代表失败
	ErrorMsg string      `json:"errorMsg,omitempty"`
	Data     interface{} `json:"data,omitempty"`
}

func TestArchiveCenterHttpClient_GetBlockByHeight(t *testing.T) {
	type fields struct {
		config *ArchiveCenterConfig
		logger utils.Logger
	}
	type args struct {
		height uint64
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    *commonPb.BlockInfo
		wantErr bool
	}{
		{
			name: "test1",
			fields: fields{
				config: &ArchiveCenterConfig{
					ChainGenesisHash:     "Genesis0",
					ArchiveCenterHttpUrl: "http://localhost",
					ReqeustSecondLimit:   10,
					RpcAddress:           "",
					TlsEnable:            false,
					Tls:                  utils.TlsConfig{},
					MaxSendMsgSize:       0,
					MaxRecvMsgSize:       0,
				},
				logger: test.NewTestLogger(t),
			},
			want: &commonPb.BlockInfo{
				Block: &commonPb.Block{
					Header: &commonPb.BlockHeader{BlockHeight: 1},
				},
			},
			args: args{height: 1},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ts := mockHttpServer(t, "/get_block_info_by_height", tt.want)
			defer ts.Close()
			tt.fields.config.ArchiveCenterHttpUrl = ts.URL
			c := &ArchiveCenterHttpClient{
				config: tt.fields.config,
				logger: tt.fields.logger,
			}
			got, err := c.GetBlockByHeight(tt.args.height, true)
			if (err != nil) != tt.wantErr {
				t.Errorf("GetBlockByHeight() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			assert.NotNil(t, got)
			if !reflect.DeepEqual(got.Block.Header.BlockHeight, tt.want.Block.Header.BlockHeight) {
				t.Errorf("GetBlockByHeight() got = %v, want %v", got, tt.want)
			}
		})
	}
}
func mockHttpServer(t *testing.T, path string, data interface{}) *httptest.Server {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Expected 'POST' request, got '%s'", r.Method)
		}
		if r.URL.EscapedPath() != path {
			t.Errorf("Expected request to '%s', got '%s'", path, r.URL.EscapedPath())
		}
		w.WriteHeader(http.StatusOK)
		resp := &Response{
			Code:     0,
			ErrorMsg: "",
			Data:     data,
		}
		body, _ := json.Marshal(resp)
		w.Write(body)
		//r.ParseForm()
		//topic := r.Form.Get("addr")
		//if topic != "shanghai" {
		//	t.Errorf("Expected request to have 'addr=shanghai', got: '%s'", topic)
		//}
	}))
	return ts
}

func TestArchiveCenterHttpClient_GetTxByTxId(t *testing.T) {
	type fields struct {
		config *ArchiveCenterConfig
		logger utils.Logger
	}
	type args struct {
		txId string
	}
	tests := []struct {
		name    string
		fields  fields
		args    args
		want    *commonPb.TransactionInfo
		wantErr bool
	}{
		{
			name: "test2",
			fields: fields{
				config: &ArchiveCenterConfig{
					ChainGenesisHash:     "Genesis0",
					ArchiveCenterHttpUrl: "http://localhost",
					ReqeustSecondLimit:   10,
					RpcAddress:           "",
					TlsEnable:            false,
					Tls:                  utils.TlsConfig{},
					MaxSendMsgSize:       0,
					MaxRecvMsgSize:       0,
				},
				logger: test.NewTestLogger(t),
			},
			want: &commonPb.TransactionInfo{
				Transaction: &commonPb.Transaction{
					Payload: &commonPb.Payload{TxId: "tx1"},
				},
			},
			args: args{txId: "tx1"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ts := mockHttpServer(t, "/get_transaction_info_by_txid", tt.want)
			defer ts.Close()
			tt.fields.config.ArchiveCenterHttpUrl = ts.URL
			c := &ArchiveCenterHttpClient{
				config: tt.fields.config,
				logger: tt.fields.logger,
			}
			got, err := c.GetTxByTxId(tt.args.txId)
			if (err != nil) != tt.wantErr {
				t.Errorf("GetTxByTxId() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			assert.NotNil(t, got)
			if !reflect.DeepEqual(got.Transaction.Payload.TxId, tt.want.Transaction.Payload.TxId) {
				t.Errorf("GetTxByTxId() got = %v, want %v", got, tt.want)
			}
		})
	}
}
