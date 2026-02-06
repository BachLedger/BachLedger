package accesscontrol

import (
	"crypto/x509"

	bcx509 "chainmaker.org/chainmaker/common/v2/crypto/x509"
	"chainmaker.org/chainmaker/pb-go/v2/config"
)

const CONSENSUS = "consensus"

func (cp *certACProvider) messageChainConfig(chainConfig *config.ChainConfig, fromMaxBFT bool) {
	cp.acService.hashType = chainConfig.GetCrypto().GetHash()
	cp.acService.initResourcePolicy(chainConfig.ResourcePolicies, cp.localOrg.id)

	updateTrustRootAndMemberFunc := func() {
		err := cp.initTrustRootsForUpdatingChainConfig(chainConfig, cp.localOrg.id)
		if err != nil {
			cp.acService.log.Error(err)
			return
		}

		cp.opts.KeyUsages = make([]x509.ExtKeyUsage, 1)
		cp.opts.KeyUsages[0] = x509.ExtKeyUsageAny

		cp.acService.memberCache.Clear()
		cp.certCache.Clear()
		err = cp.initTrustMembers(chainConfig.TrustMembers)
		if err != nil {
			cp.acService.log.Error(err)
			return
		}
	}

	//if consensus is maxbft, delay update
	if fromMaxBFT {
		updateTrustRootAndMemberFunc()
	}

}

// nolint: unused
func (cp *certACProvider) isConsensusAKI(aki string) {
	//检查aki是否对应共识节点，需要获取共识节点证书列表
	//func (cp *certACProvider) checkCRL(certChain []*bcx509.Certificate) error {
	//	if len(certChain) < 1 {
	//	return fmt.Errorf("given certificate chain is empty")
	//}
	//
	//	for _, cert := range certChain {
	//	akiCert := cert.AuthorityKeyId
	//
	//	crl, ok := cp.crl.Load(string(akiCert))
	//	if ok {
	//	// we have ac CRL, check whether the serial number is revoked
	//	for _, rc := range crl.(*pkix.CertificateList).TBSCertList.RevokedCertificates {
	//	if rc.SerialNumber.Cmp(cert.SerialNumber) == 0 {
	//	return errors.New("certificate is revoked")
	//}

	//nodeList := cp.chainConfig.Consensus.Nodes
}

func isConsensusCert(raw interface{}) bool {
	switch certInfo := raw.(type) {
	case *bcx509.Certificate:
		if len(certInfo.Subject.OrganizationalUnit) != 0 &&
			certInfo.Subject.OrganizationalUnit[0] == CONSENSUS {
			return true
		}
	case []byte:
		cert, err := bcx509.ParseCertificate(certInfo)
		if err != nil {
			return false
		}
		if len(cert.Subject.OrganizationalUnit) != 0 &&
			cert.Subject.OrganizationalUnit[0] == CONSENSUS {
			return true
		}
	}
	return false
}
