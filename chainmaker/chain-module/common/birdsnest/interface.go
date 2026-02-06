/*
Copyright (C) BABEC. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

// Package birdsnest interface
package birdsnest

type Serializer interface {
	Serialize() error
	Deserialize() error
}

// BirdsNest Bird's Nest
type BirdsNest interface {
	GetHeight() uint64
	SetHeight(height uint64)
	// Add the key
	Add(key Key) error
	// Adds adding Multiple Keys
	Adds(keys []Key) (result error)
	// AddsAndSetHeight Adds and SetHeight
	AddsAndSetHeight(keys []Key, height uint64) (result error)
	// Contains the key
	Contains(key Key, rules ...RuleType) (bool, error)
	ValidateRule(key Key, rules ...RuleType) error
	// Info Current cuckoos nest information and status
	Info() []uint64

	Start()
}

type CuckooFilter interface {
	IsFull() bool
	Add(key Key) (bool, error)
	Contains(key Key) (bool, error)
	Encode() (FilterEncoder, error)
	Extension() FilterExtension
	Info() []uint64
}

// FilterExtension filter extension
type FilterExtension interface {
	// Validate validate key
	Validate(key Key, full bool) error
	Store(key Key) error
	Serialize() []byte
}

type Snapshot interface {
	Write(data []byte) error
	Read() ([]byte, error)
}

type Logger interface {
	Debugf(format string, args ...interface{})
	Errorf(format string, args ...interface{})
	Infof(format string, args ...interface{})
	Warnf(format string, args ...interface{})
}

// BirdsNestSerialize Bird's nest serialize
type BirdsNestSerialize struct {
	// Bird's Nest config
	Config *BirdsNestConfig `protobuf:"bytes,1,opt,name=config,proto3" json:"config,omitempty"`
	// The final height
	Height uint64 `protobuf:"varint,2,opt,name=height,proto3" json:"height,omitempty"`
	// current index
	CurrentIndex uint32 `protobuf:"varint,3,opt,name=currentIndex,proto3" json:"currentIndex,omitempty"`
	// A group of cuckoos filter
	Filters []*CuckooFilterSerialize `protobuf:"bytes,4,rep,name=filters,proto3" json:"filters,omitempty"`
}

type CuckooFilterSerialize struct {
	// The field "cuckoo" is used to hold the serialized data of the cuckoo
	// Pb limit: The size of bytes cannot be larger than 4 GB
	Cuckoo []byte `protobuf:"bytes,1,opt,name=cuckoo,proto3" json:"cuckoo,omitempty"`
	// Carries the ID of the time
	Extension []byte `protobuf:"bytes,2,opt,name=extension,proto3" json:"extension,omitempty"`
	// cuckoo configuration
	Config []byte `protobuf:"bytes,3,opt,name=config,proto3" json:"config,omitempty"`
}
