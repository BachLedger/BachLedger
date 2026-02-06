package mock

import vmPb "chainmaker.org/chainmaker/pb-go/v2/vm"

func CalcStringListDataSize(list []string) int {
	size := 0
	for _, data := range list {
		size += len(data)
	}

	return size
}

func CalcBytesMapDataSize(params map[string][]byte) int {
	size := 0
	for key, val := range params {
		size += len(key) + len(val)
	}

	return size
}

func CalcBatchKeysParamsSize(keys []*vmPb.BatchKey) int {
	size := 0
	for _, key := range keys {
		size += len(key.ContractName) + len(key.Key) + len(key.Field)
	}
	return size
}

func CalcBatchKeysReturnsSize(keys []*vmPb.BatchKey) int {
	size := 0
	for _, key := range keys {
		size += len(key.Value)
	}
	return size
}
