/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package linkedhashmap

import "container/list"

type hashMapNode struct {
	linklistNode *list.Element
	val          interface{}
}

type LinkedHashMap struct {
	linklist *list.List
	hashmap  map[string]interface{}
}

func NewLinkedHashMap() *LinkedHashMap {
	return &LinkedHashMap{
		linklist: list.New(),
		hashmap:  make(map[string]interface{}),
	}
}

func (linkMap *LinkedHashMap) Add(key string, val interface{}) bool {
	if _, isExists := linkMap.hashmap[key]; isExists {
		return false
	}

	linkListNode := linkMap.linklist.PushBack(key)
	linkMap.hashmap[key] = &hashMapNode{
		linklistNode: linkListNode,
		val:          val,
	}
	return true
}

func (linkMap *LinkedHashMap) Get(key string) interface{} {
	originLinkedHashMapNode, isExists := linkMap.hashmap[key]
	if !isExists {
		return nil
	}
	return (originLinkedHashMapNode.(*hashMapNode)).val
}

func (linkMap *LinkedHashMap) Size() int {
	return len(linkMap.hashmap)
}
func (linkMap *LinkedHashMap) Remove(key string) (bool, interface{}) {
	originLinkedHashMapNode, isExists := linkMap.hashmap[key]
	if !isExists {
		return false, nil
	}

	linkedHashMapNode, _ := originLinkedHashMapNode.(*hashMapNode)
	delete(linkMap.hashmap, key)
	linkMap.linklist.Remove(linkedHashMapNode.linklistNode)
	return true, linkedHashMapNode.val
}

func (linkMap *LinkedHashMap) GetLinkList() *list.List {
	return linkMap.linklist
}
