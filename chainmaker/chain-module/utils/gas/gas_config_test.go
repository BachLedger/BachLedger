package gas

import (
	"fmt"
	"math"
	"testing"
)

func TestGasPrice(t *testing.T) {
	gasPrice := float32(0.0)
	var dataSize uint64 = math.MaxUint64

	fmt.Printf("gas = %v \n", uint64(float64(gasPrice)*float64(dataSize)))

	fmt.Printf("max float64 = %v \n", math.MaxFloat64)
	fmt.Printf("max uint64 = %v \n", uint64(0xF800000000000000))
}
