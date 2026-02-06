<!--
Guiding Principles:

Changelogs are for humans, not machines.
There should be an entry for every single version.
The same types of changes should be grouped.
Versions and sections should be linkable.
The latest version comes first.
The release date of each version is displayed.
Mention whether you follow Semantic Versioning.

Usage:

Change log entries are to be added to the Unreleased section under the
appropriate stanza (see below). Each entry should ideally include a tag and
the Github issue reference in the following format:

* (<tag>) \#<issue-number> message

The issue numbers will later be link-ified during the release process so you do
not have to worry about including a link manually, but you can if you wish.

Types of changes (Stanzas):

"Features" for new features.
"Improvements" for changes in existing functionality.
"Deprecated" for soon-to-be removed features.
"Bug Fixes" for any bug fixes.
"API Breaking" for breaking exported APIs used by developers building on SDK.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

### Features

### Improvements

### API Breaking Changes

### Bug Fixes

### Deprecated

## v2.3.2

### Features

##### 归档系列API 兼容了`归档中心`作为链下存储
##### 获取节点归档状态详细信息
```
GetArchiveStatus
```

##### 发送合约管理请求（创建、更新、冻结、解冻、吊销）
```
SendContractManageRequestWithPayer
```

##### 发起多签请求(指定gas代扣账户)
```
MultiSignContractReqWithPayer
```

##### 触发执行多签请求(指定gas代扣账户)
```
MultiSignContractTrigWithPayer
```

##### 根据发起多签请求所需的参数构建payload
```
CreateMultiSignReqPayloadWithGasLimit
```

##### 发起多签投票
```
MultiSignContractVoteWithGasLimit
```

##### 发起多签投票(指定gas代扣账户)
```
MultiSignContractVoteWithGasLimitAndPayer
```

##### 根据txId查询多签状态
```
MultiSignContractQueryWithParams
```

##### 设置链配置的 default gas_price 参数
```
CreateSetInvokeGasPricePayload
```

##### 设置链配置的 install_base gas 参数
```
CreateSetInstallBaseGasPayload
```

##### 设置链配置的 install gas_price 参数
```
CreateSetInstallGasPricePayload
```

## v2.3.1

### Features

##### 发起多签请求
**参数说明**
- payload: 待签名payload
- endorsers: 背书签名信息列表
- timeout: 超时时间，单位：s，若传入-1，将使用默认超时时间：10s
- withSyncResult: 是否同步获取交易执行结果
  当为true时，若成功调用，common.TxResponse.ContractResult.Result为common.TransactionInfo
  当为false时，若成功调用，common.TxResponse.ContractResult为空，可以通过common.TxResponse.TxId查询交易结果
```go
	MultiSignContractReq(payload *common.Payload, endorsers []*common.EndorsementEntry, timeout int64,
		withSyncResult bool) (*common.TxResponse, error)
```

##### 发起多签投票
**参数说明**
- payload: 待签名payload
- endorser: 投票人对多签请求 payload 的签名信息
- isAgree: 投票人对多签请求是否同意，true为同意，false则反对
- timeout: 超时时间，单位：s，若传入-1，将使用默认超时时间：10s
- withSyncResult: 是否同步获取交易执行结果
  当为true时，若成功调用，common.TxResponse.ContractResult.Result为common.TransactionInfo
  当为false时，若成功调用，common.TxResponse.ContractResult为空，可以通过common.TxResponse.TxId查询交易结果
```go
	MultiSignContractVote(payload *common.Payload, endorser *common.EndorsementEntry, isAgree bool,
		timeout int64, withSyncResult bool) (*common.TxResponse, error)
```

##### 触发执行多签请求
**参数说明**
- payload: 待签名payload
- timeout: 超时时间，单位：s，若传入-1，将使用默认超时时间：10s
  //	 - limit: 本次执行多签请求支付的 gas 上限
- withSyncResult: 是否同步获取交易执行结果
  当为true时，若成功调用，common.TxResponse.ContractResult.Result为common.TransactionInfo
  当为false时，若成功调用，common.TxResponse.ContractResult为空，可以通过common.TxResponse.TxId查询交易结果
```go
	MultiSignContractTrig(multiSignReqPayload *common.Payload,
		timeout int64, limit *common.Limit, withSyncResult bool) (*common.TxResponse, error)
```

## v2.3.0 bugfix

## v2.2.1 bugfix

## v2.2.0 - 2021-12-17

### Features

* (gas) 新增启用/停用gas计费开关API
* (gas) 新增 attach `Limit` API
* (通用) 新增修改地址类型API
* (通用) 提供至信链地址生成相关API
* (Grpc client) grpc客户端发送消息时，可设置允许单条message大小的最大值(MB)

### Improvements

* (订阅) 支持订阅断线自动重连机制
* (订阅) 支持合约事件按照区块高度订阅历史事件
