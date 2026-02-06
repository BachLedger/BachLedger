/*
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package testframework

import (
	"fmt"
	"io/ioutil"
	"path/filepath"

	"chainmaker.org/chainmaker/logger/v2"
	netPb "chainmaker.org/chainmaker/pb-go/v2/net"

	"chainmaker.org/chainmaker/common/v2/msgbus"
	net2 "chainmaker.org/chainmaker/consensus-utils/v2/testframework/net"
	"chainmaker.org/chainmaker/protocol/v2"
)

// NewNetService create protocol.NetService
func NewNetService(net protocol.Net,
	chainId string,
	ac protocol.AccessControlProvider,
	chainConf protocol.ChainConf,
	msgBus msgbus.MessageBus) (protocol.NetService, error) {
	var nsf net2.NetServiceFactory

	ns, err := nsf.NewNetService(
		net,
		chainId,
		ac,
		chainConf,
		net2.WithConsensusNodeUid(memberIds...),
		net2.WithMsgBus(msgBus),
	)
	if err != nil {
		return nil, err
	}
	return ns, nil
}

// NewNet create protocol.Net
func NewNet(keyPath, certPath string, config *TestNodeConfig) (protocol.Net, error) {
	var netFactory net2.NetFactory
	var err error

	//prepare cert key path
	if !filepath.IsAbs(keyPath) {
		keyPath, err = filepath.Abs(keyPath)
		if err != nil {
			return nil, err
		}
	}
	//prepare cert file path
	if !filepath.IsAbs(certPath) {
		certPath, err = filepath.Abs(certPath)
		if err != nil {
			return nil, err
		}
	}

	net, _ := netFactory.NewNet(
		protocol.Libp2p,
		net2.WithListenAddr(config.ListenAddr),
		net2.WithReadySignalC(readyC),
		net2.WithCrypto(false, keyPath, certPath),
		net2.WithSeeds(config.Seeds...),
	)

	// 添加AC
	net.AddAC(BlockchainId, config.Ac)
	//添加TRustRoot
	roots := make([][]byte, 0, len(config.Seeds))
	for i := 0; i < len(config.Seeds); i++ {
		absPath, _ := filepath.Abs(fmt.Sprintf(rootFilePrefix, org_s[i]))
		root, _ := ioutil.ReadFile(absPath)
		roots = append(roots, root)
	}
	net.SetChainCustomTrustRoots(BlockchainId, roots)

	return net, nil
}

// ####################################################################################################################
//                                                  Net Engine for cluster mode
// ####################################################################################################################

// NetEngineForTest struct
type NetEngineForTest struct {
	chainId string
	nodeId  string
	msgBus  msgbus.MessageBus
	tf      *TestClusterFramework
	log     *logger.CMLogger
}

// NewNetEngineForTest new net engine
func NewNetEngineForTest(chainId, nodeId string, msgBus msgbus.MessageBus, tf *TestClusterFramework) *NetEngineForTest {
	ne := &NetEngineForTest{
		chainId: chainId,
		nodeId:  nodeId,
		msgBus:  msgBus,
		tf:      tf,
		log:     logger.GetLogger(logger.MODULE_NET),
	}
	return ne
}

// Start engine
func (ne *NetEngineForTest) Start() {
	fmt.Printf("[nodeId:%d,%s] Net Engine Start\n", map_nodeId_num[ne.nodeId], ne.nodeId)
	ne.log.Infof("[nodeId:%d,%s] Net Engine Start", map_nodeId_num[ne.nodeId], ne.nodeId)
	ne.msgBus.Register(msgbus.SendConsensusMsg, ne)
}

// Stop engine
func (ne *NetEngineForTest) Stop() {
	ne.log.Infof("[nodeId:%d,%s] Net Engine Stop", map_nodeId_num[ne.nodeId], ne.nodeId)
	fmt.Printf("[nodeId:%d,%s] Net Engine Stop\n", map_nodeId_num[ne.nodeId], ne.nodeId)
}

// OnMessage NetEngineForTest
func (ne *NetEngineForTest) OnMessage(message *msgbus.Message) {
	switch message.Topic {
	case msgbus.SendConsensusMsg:
		if netMsg, ok := message.Payload.(*netPb.NetMsg); ok {
			if netMsg.Type.String() != netPb.NetMsg_CONSENSUS_MSG.String() {
				ne.log.Infof("[nodeId:%d,%s] net msg type is not expected, actual:%s, expected:%s\n",
					map_nodeId_num[ne.nodeId], ne.nodeId, netMsg.Type.String(), netPb.NetMsg_CONSENSUS_MSG.String())
				return
			}
			// broadcast or send
			if netMsg.To == "" {
				ne.log.Infof("[nodeId:%d,%s] broadcast net msg", map_nodeId_num[ne.nodeId], ne.nodeId)
				for _, tn := range ne.tf.tns {
					if ne.nodeId != tn.nodeId {
						tn.msgBus.Publish(msgbus.RecvConsensusMsg, netMsg)
					}
				}
			} else {
				for _, tn := range ne.tf.tns {
					if netMsg.To == tn.nodeId {
						ne.log.Infof("[nodeId:%d,%s] send net msg to [nodeId:%d,%s]",
							map_nodeId_num[ne.nodeId], ne.nodeId, map_nodeId_num[netMsg.To], netMsg.To)
						tn.msgBus.Publish(msgbus.RecvConsensusMsg, netMsg)
					}
				}
			}
		}
	}
}

// OnQuit NetEngineForTest
func (ne *NetEngineForTest) OnQuit() {
	ne.log.Infof("[nodeId:%d,%s] Net Engine quit", map_nodeId_num[ne.nodeId], ne.nodeId)
}
