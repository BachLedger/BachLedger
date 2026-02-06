/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package net

import (
	"errors"
	"io/ioutil"

	libp2p "chainmaker.org/chainmaker/net-libp2p/libp2pnet"
	"chainmaker.org/chainmaker/protocol/v2"
)

var ErrorNetType = errors.New("error net type")

// NetFactory provide a way to create net instance.
type NetFactory struct {
	netType protocol.NetType

	n protocol.Net
}

// NetOption is a function apply options to net instance.
type NetOption func(cfg *NetFactory) error

func WithReadySignalC(signalC chan struct{}) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetReadySignalC(signalC)
		}
		return nil
	}
}

// WithListenAddr set addr that the local net will listen on.
func WithListenAddr(addr string) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetListenAddr(addr)
		}
		return nil
	}
}

// WithCrypto set private key file and tls cert file for the net to create connection.
func WithCrypto(pkMode bool, keyFile, certFile string, encKeyFile, encCertFile string) NetOption {
	return func(nf *NetFactory) error {
		var (
			err                       error
			keyBytes, certBytes       []byte
			encKeyBytes, encCertBytes []byte
		)
		//try to read
		encKeyBytes, _ = ioutil.ReadFile(encKeyFile)
		encCertBytes, _ = ioutil.ReadFile(encCertFile)

		keyBytes, err = ioutil.ReadFile(keyFile)
		if err != nil {
			return err
		}
		if !pkMode {
			certBytes, err = ioutil.ReadFile(certFile)
			if err != nil {
				return err
			}
		}
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPubKeyModeEnable(pkMode)
			n.Prepare().SetKey(keyBytes)
			if !pkMode {
				n.Prepare().SetCert(certBytes)
				n.Prepare().SetEncKey(encKeyBytes)
				n.Prepare().SetEncCert(encCertBytes)
			}
		}
		return nil
	}
}

// WithSeeds set addresses of discovery service node.
func WithSeeds(seeds ...string) NetOption {
	return func(nf *NetFactory) error {
		if seeds == nil {
			return nil
		}
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			for _, seed := range seeds {
				n.Prepare().AddBootstrapsPeer(seed)
			}
		}
		return nil
	}
}

// WithPeerStreamPoolSize set the max stream pool size for every node that connected to us.
func WithPeerStreamPoolSize(size int) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPeerStreamPoolSize(size)
		}
		return nil
	}
}

// WithPubSubMaxMessageSize set max message size (M) for pub/sub.
func WithPubSubMaxMessageSize(size int) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPubSubMaxMsgSize(size)
		}
		return nil
	}
}

// WithMaxPeerCountAllowed set max count of nodes that connected to us.
func WithMaxPeerCountAllowed(max int) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetMaxPeerCountAllow(max)
		}
		return nil
	}
}

// WithMaxConnCountAllowed set max count of connections for each peer that connected to us.
func WithMaxConnCountAllowed(max int) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			// not supported
		}
		return nil
	}
}

// WithPeerEliminationStrategy set the strategy for eliminating node when the count of nodes
// that connected to us reach the max value.
func WithPeerEliminationStrategy(strategy int) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPeerEliminationStrategy(strategy)
		}
		return nil
	}
}

// WithBlackAddresses set addresses of the nodes for blacklist.
func WithBlackAddresses(blackAddresses ...string) NetOption {
	return func(nf *NetFactory) error {
		if blackAddresses == nil {
			return nil
		}
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			for _, ba := range blackAddresses {
				n.Prepare().AddBlackAddress(ba)
			}
		}
		return nil
	}
}

// WithBlackNodeIds set ids of the nodes for blacklist.
func WithBlackNodeIds(blackNodeIds ...string) NetOption {
	return func(nf *NetFactory) error {
		if blackNodeIds == nil {
			return nil
		}
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			for _, bn := range blackNodeIds {
				n.Prepare().AddBlackPeerId(bn)
			}
		}
		return nil
	}
}

// WithMsgCompression set whether compressing the payload when sending msg.
func WithMsgCompression(enable bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.SetCompressMsgBytes(enable)
		}
		return nil
	}
}

func WithInsecurity(isInsecurity bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetIsInsecurity(isInsecurity)
		}
		return nil
	}
}

func WithPktEnable(pktEnable bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPktEnable(pktEnable)
		}
		return nil
	}
}

// WithPriorityControlEnable config priority controller
func WithPriorityControlEnable(priorityCtrlEnable bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			n, _ := nf.n.(*libp2p.LibP2pNet)
			n.Prepare().SetPriorityCtrlEnable(priorityCtrlEnable)
		}
		return nil
	}
}

// NewNet create a new net instance.
func (nf *NetFactory) NewNet(netType protocol.NetType, opts ...NetOption) (protocol.Net, error) {
	nf.netType = netType
	switch nf.netType {
	case protocol.Libp2p:
		localNet, err := libp2p.NewLibP2pNet(GlobalNetLogger)
		if err != nil {
			return nil, err
		}
		nf.n = localNet
	default:
		return nil, ErrorNetType
	}
	if err := nf.Apply(opts...); err != nil {
		return nil, err
	}
	return nf.n, nil
}

// Apply options.
func (nf *NetFactory) Apply(opts ...NetOption) error {
	for _, opt := range opts {
		if opt == nil {
			continue
		}
		if err := opt(nf); err != nil {
			return err
		}
	}
	return nil
}

// WithStunClient read stun client cfg
// clientListenAddr: listen bind addr
// stunServerAddr: stun server addr
// networkType: udp,tcp,quic
func WithStunClient(clientListenAddr, stunServerAddr, networkType string, enable bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			// not supported
		}
		return nil
	}
}

// WithStunServer read stun server cfg
// enable: set stun server if enable
// twoPublicAddr: one device have two PublicAddr
// addr1, addr2 must set
func WithStunServer(enable, twoPublicAddr bool, other string, notifyAddr, localNotify,
	addr1, addr2, addr3, addr4, networkType string) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			// not supported
		}
		return nil
	}
}

// WithHolePunch read hole-punch cfg
// enable: set hole-punch function if enable
func WithHolePunch(enable bool) NetOption {
	return func(nf *NetFactory) error {
		switch nf.netType {
		case protocol.Libp2p:
			// not supported
		}
		return nil
	}
}
