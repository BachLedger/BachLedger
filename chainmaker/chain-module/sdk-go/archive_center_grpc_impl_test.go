package chainmaker_sdk_go

import (
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"testing"

	archivePb "chainmaker.org/chainmaker/pb-go/v2/archivecenter"
	commonPb "chainmaker.org/chainmaker/pb-go/v2/common"
	"chainmaker.org/chainmaker/pb-go/v2/mock"
	"chainmaker.org/chainmaker/protocol/v2/test"
	"chainmaker.org/chainmaker/sdk-go/v2/utils"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc"
	"google.golang.org/grpc/test/bufconn"
)

const bufSize = 1024 * 1024

var lis *bufconn.Listener

func InitGrpcServer(t *testing.T, svr *mock.MockArchiveCenterServerServer) {
	lis = bufconn.Listen(bufSize)
	s := grpc.NewServer()
	archivePb.RegisterArchiveCenterServerServer(s, svr)
	go func() {
		if err := s.Serve(lis); err != nil {
			log.Fatalf("Server exited with error: %v", err)
		}
	}()
}
func bufDialer(context.Context, string) (net.Conn, error) {
	return lis.Dial()
}
func initUTGrpcClient(t *testing.T) *ArchiveCenterGrpcClient {
	var retClient ArchiveCenterGrpcClient
	retClient.maxSendSize = 100 * 1024 * 1024
	retClient.logger = test.NewTestLogger(t)
	ctx := context.Background()
	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(bufDialer), grpc.WithInsecure())
	if err != nil {
		t.Fatalf("Failed to dial bufnet: %v", err)
	}
	retClient.conn = conn
	retClient.config = &ArchiveCenterConfig{
		ChainGenesisHash:     "Genesis0",
		ArchiveCenterHttpUrl: "",
		ReqeustSecondLimit:   10,
		RpcAddress:           "",
		TlsEnable:            false,
		Tls:                  utils.TlsConfig{},
		MaxSendMsgSize:       0,
		MaxRecvMsgSize:       0,
	}
	retClient.client = archivePb.NewArchiveCenterServerClient(conn)
	return &retClient
}

func TestArchiveCenterGrpcGetBlockByHeight(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	svr.EXPECT().GetBlockByHeight(gomock.Any(), gomock.Any()).Return(
		&archivePb.BlockWithRWSetResp{
			BlockData: &commonPb.BlockInfo{
				Block:     &commonPb.Block{Header: &commonPb.BlockHeader{BlockHeight: 123}},
				RwsetList: nil,
			},
			Code:    0,
			Message: "",
		}, nil)

	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	resp, err := client.GetBlockByHeight(123, true)
	if err != nil {
		t.Fatalf("GetBlockByHeight failed: %v", err)
	}
	t.Logf("Response: %+v", resp)
	assert.EqualValues(t, 123, resp.Block.Header.BlockHeight)
}

func TestArchiveCenterGrpcGetBlockByHash(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	svr.EXPECT().GetBlockByHash(gomock.Any(), gomock.Any()).Return(
		&archivePb.BlockWithRWSetResp{
			BlockData: &commonPb.BlockInfo{
				Block:     &commonPb.Block{Header: &commonPb.BlockHeader{BlockHeight: 123, BlockHash: []byte("hash1")}},
				RwsetList: nil,
			},
			Code:    0,
			Message: "",
		}, nil)

	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	resp, err := client.GetBlockByHash("hash1", true)
	if err != nil {
		t.Fatalf("GetBlockByHeight failed: %v", err)
	}
	t.Logf("Response: %+v", resp)
	assert.EqualValues(t, 123, resp.Block.Header.BlockHeight)
}

func TestArchiveCenterGrpcGetBlockByTxId(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	svr.EXPECT().GetBlockByTxId(gomock.Any(), gomock.Any()).Return(
		&archivePb.BlockWithRWSetResp{
			BlockData: &commonPb.BlockInfo{
				Block:     &commonPb.Block{Header: &commonPb.BlockHeader{BlockHeight: 123, BlockHash: []byte("hash1")}},
				RwsetList: nil,
			},
			Code:    0,
			Message: "",
		}, nil)

	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	resp, err := client.GetBlockByTxId("tx1", true)
	if err != nil {
		t.Fatalf("GetBlockByHeight failed: %v", err)
	}
	t.Logf("Response: %+v", resp)
	assert.EqualValues(t, 123, resp.Block.Header.BlockHeight)
}

func TestArchiveCenterGrpcGetTxByTxId(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	svr.EXPECT().GetTxByTxId(gomock.Any(), gomock.Any()).Return(
		&archivePb.TransactionResp{
			Transaction: &commonPb.Transaction{Payload: &commonPb.Payload{TxId: "tx123"}},
			Code:        0,
			Message:     "",
		}, nil)
	svr.EXPECT().GetTxDetailByTxId(gomock.Any(), gomock.Any()).Return(&archivePb.TxDetailByIdResp{
		Height:          123,
		TxExist:         true,
		TxConfirmedTime: 11110,
		Code:            0,
		Message:         "",
	}, nil)
	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	resp, err := client.GetTxByTxId("tx1")
	if err != nil {
		t.Fatalf("GetBlockByHeight failed: %v", err)
	}
	t.Logf("Response: %+v", resp)
	assert.EqualValues(t, "tx123", resp.Transaction.Payload.TxId)
}

func TestArchiveCenterGrpcRegister(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	svr.EXPECT().GetArchivedStatus(gomock.Any(), gomock.Any()).Return(
		&archivePb.ArchiveStatusResp{
			ArchivedHeight: 0,
			InArchive:      false,
			Code:           1,
			Message:        "chain genesis not exists",
		}, nil)

	serverLog := ""
	svr.EXPECT().Register(gomock.Any(), gomock.Any()).DoAndReturn(func(ctx context.Context,
		req *archivePb.ArchiveBlockRequest) (*archivePb.RegisterResp, error) {
		serverLog += fmt.Sprintf("Unique:%s,Block:%#v", req.ChainUnique, req.Block.Block)
		return &archivePb.RegisterResp{
			RegisterStatus: 0,
			Code:           0,
			Message:        "OK",
		}, nil
	})

	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	err := client.Register(&commonPb.BlockInfo{
		Block: &commonPb.Block{
			Header: &commonPb.BlockHeader{
				BlockHash: []byte("hash10"),
			}}})
	if err != nil {
		t.Fatalf("GetBlockByHeight failed: %v", err)
	}
	t.Logf("Response: %+v", serverLog)
}

func TestArchiveCenterGrpcArchiveBlocks(t *testing.T) {
	ctl := gomock.NewController(t)
	defer ctl.Finish()
	svr := mock.NewMockArchiveCenterServerServer(ctl)
	archived := 0
	svr.EXPECT().ArchiveBlocks(gomock.Any()).Do(
		func(s archivePb.ArchiveCenterServer_ArchiveBlocksServer) error {
			for {
				tempRequest, tempRequestError := s.Recv()
				if tempRequestError != nil {
					if tempRequestError == io.EOF {
						// 说明客户端已经发送完了数据
						t.Log("ArchiveBlocks got EOF")
						break
					}
					t.Fatalf("ArchiveBlocks recv got error %s ", tempRequestError.Error())
					return tempRequestError
				}
				t.Logf("Receive Request Block:%#v", tempRequest)
				archived++
				sendError := s.Send(&archivePb.ArchiveBlockResp{
					ArchiveStatus: 2,
					Code:          0,
					Message:       "OK",
				})
				if sendError != nil {
					return sendError
				}

			}
			return nil
		}).AnyTimes()
	InitGrpcServer(t, svr)
	client := initUTGrpcClient(t)
	defer client.conn.Close()
	blks := mockBlocks(t)
	err := client.ArchiveBlocks(blks, blks.heightNotice)
	if err != nil {
		t.Fatalf("ArchiveBlocks failed: %v", err)
	}
	assert.EqualValues(t, 5, archived)
}

func mockBlocks(t *testing.T) *blocks {
	blks := &blocks{
		queryFunc: func(blockHeight uint64, withRWSet bool) (*commonPb.BlockInfo, error) {
			return &commonPb.BlockInfo{Block: &commonPb.Block{Header: &commonPb.BlockHeader{BlockHeight: blockHeight}}}, nil
		},
		height:    0,
		endHeight: 5,
		total:     5,
		heightNotice: func(msg ProcessMessage) error {
			t.Logf("%#v", msg)
			return nil
		},
	}
	return blks
}
