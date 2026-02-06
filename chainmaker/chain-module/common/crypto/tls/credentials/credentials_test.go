package credentials

import (
	"crypto/tls"
	"log"
	"net"
	"testing"
	"time"

	"chainmaker.org/chainmaker/common/v2/crypto/tls/config"

	cmtls "chainmaker.org/chainmaker/common/v2/crypto/tls"
	"chainmaker.org/chainmaker/common/v2/crypto/tls/credentials/helloworld"
	"github.com/stretchr/testify/require"
	"golang.org/x/net/context"
	"google.golang.org/grpc"
)

const (
	requestMsg  = "hello, I'm client"
	responseMsg = "hi, I'm server"
)

type server struct{}

func (s *server) SayHello(ctx context.Context, req *helloworld.HelloRequest) (*helloworld.HelloReply, error) {
	log.Printf("Received %s", req.Name)
	return &helloworld.HelloReply{Message: responseMsg}, nil
}

// tls certs
const (
	ca         = "testdata/cacert.pem"
	serverCert = "testdata/servercert.pem"
	serverKey  = "testdata/serverkey.pem"
	userCert   = "testdata/usercert.pem"
	userKey    = "testdata/userkey.pem"
)

// grpc server
func serverRun(t *testing.T, port string, tlsVersion uint16) {
	cfg, err := config.GetConfig(serverCert, serverKey, ca, true)
	cfg.ClientAuth = cmtls.RequireAndVerifyClientCert
	cfg.MaxVersion = tlsVersion

	creds := NewTLS(cfg)
	s := grpc.NewServer(grpc.Creds(creds))
	helloworld.RegisterGreeterServer(s, &server{})

	lis, err := net.Listen("tcp", port)
	require.NoError(t, err)

	err = s.Serve(lis)
	require.NoError(t, err)
}

func clientRun(t *testing.T, address string, tlsVersion uint16, stop chan struct{}) {
	cfg, err := config.GetConfig(serverCert, serverKey, ca, false)
	cfg.ClientAuth = cmtls.RequireAndVerifyClientCert
	cfg.MaxVersion = tlsVersion
	cfg.ServerName = "chainmaker.org"

	creds := NewTLS(cfg)
	conn, err := grpc.Dial(address, grpc.WithTransportCredentials(creds))
	defer conn.Close()
	require.NoError(t, err)

	c := helloworld.NewGreeterClient(conn)
	r, err := c.SayHello(context.Background(), &helloworld.HelloRequest{Name: requestMsg})
	require.NoError(t, err)
	require.Equal(t, responseMsg, r.Message)

	stop <- struct{}{}
}

func Test_GMGrpcWithTwoWayAuth(t *testing.T) {
	stop := make(chan struct{}, 1)
	go serverRun(t, ":8090", tls.VersionTLS12)
	time.Sleep(time.Second * 3) //wait for server start
	go clientRun(t, "localhost:8090", tls.VersionTLS12, stop)
	<-stop
}

func Test_GMGrpcWithTwoWayAuth2(t *testing.T) {
	stop := make(chan struct{}, 1)
	go serverRun(t, ":8091", tls.VersionTLS13)
	time.Sleep(time.Second * 3) //wait for server start
	go clientRun(t, "localhost:8091", tls.VersionTLS13, stop)
	<-stop
}
