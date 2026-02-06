/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/
package common

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestExtensionIDEqual(t *testing.T) {
	var a []int
	var b []int
	require.True(t, extensionIDEqual(a, b))

	a = append(a, 1)
	require.False(t, extensionIDEqual(a, b))

	b = append(b, 1)
	require.True(t, extensionIDEqual(a, b))

	b = append(b, 2)
	require.False(t, extensionIDEqual(a, b))
}
