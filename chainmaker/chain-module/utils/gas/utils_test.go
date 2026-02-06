package gas

import (
	"fmt"
	"math"
	"testing"

	"github.com/shopspring/decimal"

	"github.com/stretchr/testify/assert"
)

func TestGasMultiplyOverflow(t *testing.T) {
	gas, err := MultiplyGasPrice(math.MaxInt64, 2.1)
	fmt.Printf("gas = 0x%016X \n", gas)
	fmt.Printf("err = %v", err)
	assert.NotNil(t, err)
}

func TestGasMultiplyMaxDataSize(t *testing.T) {
	fmt.Printf("max int64 = %v \n", math.MaxInt64)
	gas, err := MultiplyGasPrice(math.MaxInt64, 1.9)
	fmt.Printf("gas = 0x%016X \n", gas)
	fmt.Printf("gas = %d \n", gas)
	assert.Nil(t, err)
}

func TestMaxUint64(t *testing.T) {
	fmt.Printf("max uint64 = %v \n", maxUint64)
}

func TestFloat64(t *testing.T) {
	f := decimal.NewFromFloat32(2.1112)
	fmt.Printf("float = %v \n", f)
}

func TestCeil(t *testing.T) {
	result, err := MultiplyGasPrice(100, 2.1112)
	if err != nil {
		t.Fatalf("ceil failed, err = %v", err)
	}

	fmt.Printf("ceil float = %v \n", result)
}
