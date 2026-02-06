/*
Copyright (C) BABEC. All rights reserved.
Copyright (C) THL A29 Limited, a Tencent company. All rights reserved.

SPDX-License-Identifier: Apache-2.0
*/

package gredis

import (
	"encoding/json"
	"errors"
	"strconv"
	"time"

	"github.com/gomodule/redigo/redis"
)

type RedisHandler struct {
	RedisConn *redis.Pool
}

func NewRedisHandler() *RedisHandler {
	r := new(RedisHandler)
	return r
}

func (r *RedisHandler) Init(url, auth string, db, maxIdle, maxActive, idleTimeout int) error {
	r.RedisConn = &redis.Pool{
		MaxIdle:     maxIdle,
		MaxActive:   maxActive,
		IdleTimeout: time.Duration(idleTimeout),
		Dial: func() (redis.Conn, error) {
			c, err := redis.Dial("tcp", url)
			if err != nil {
				return nil, err
			}

			if auth != "" {
				if _, err = c.Do("AUTH", auth); err != nil {
					c.Close()
					return nil, err
				}
			}

			if db != 0 {
				if _, err = c.Do("SELECT", db); err != nil {
					c.Close()
					return nil, err
				}
			}

			return c, err
		},
		//应用程序检查健康功能
		TestOnBorrow: func(c redis.Conn, _ time.Time) error {
			_, err := c.Do("PING")
			return err
		},
	}

	_, err := r.RedisConn.Get().Do("PING")
	if err != nil {
		return err
	}

	return nil
}

// Set json
func (r *RedisHandler) Set(key string, data interface{}, time int) error {
	conn := r.RedisConn.Get()
	defer conn.Close()

	value, err := json.Marshal(data)
	if err != nil {
		return err
	}

	_, err = conn.Do("SET", key, value)
	if err != nil {
		return err
	}

	if time > 0 {
		_, err = conn.Do("EXPIRE", key, time)
		if err != nil {
			return err
		}
	}

	return nil
}

func (r *RedisHandler) SetVal(key, value interface{}, time int) error {
	conn := r.RedisConn.Get()
	defer conn.Close()

	if v, ok := value.(int); ok {
		value = strconv.Itoa(v)
	} else if v, ok := value.(string); ok {
		value = v
	} else if v, ok := value.([]byte); ok {
		value = v
	} else {
		return errors.New("redis set invalid value")
	}

	_, err := conn.Do("SET", key, value)
	if err != nil {
		return err
	}

	if time > 0 {
		_, err = conn.Do("EXPIRE", key, time)
		if err != nil {
			return err
		}
	}

	return nil
}

// Exists check a key
func (r *RedisHandler) Exists(key string) bool {
	conn := r.RedisConn.Get()
	defer conn.Close()

	exists, err := redis.Bool(conn.Do("EXISTS", key))
	if err != nil {
		return false
	}

	return exists
}

// ttl a key
func (r *RedisHandler) Ttl(key string) int {
	conn := r.RedisConn.Get()
	defer conn.Close()

	ttl, err := redis.Int(conn.Do("TTL", key))
	if err != nil {
		return -3
	}

	return ttl
}

// Get get a key
func (r *RedisHandler) Get(key string) ([]byte, error) {
	conn := r.RedisConn.Get()
	defer conn.Close()

	reply, err := redis.Bytes(conn.Do("GET", key))
	if err != nil {
		return nil, err
	}

	return reply, nil
}

func (r *RedisHandler) GetString(key string) (string, error) {
	bytes, err := r.Get(key)
	if err != nil {
		return "", err
	}

	return string(bytes), nil
}

func (r *RedisHandler) GetInt(key string) (int, error) {
	bytes, err := r.Get(key)
	if err != nil {
		return -1, err
	}

	sVal := string(bytes)

	iVal, err := strconv.Atoi(sVal)
	if err != nil {
		return -1, err
	}

	return iVal, nil
}

// Delete delete a kye
func (r *RedisHandler) Delete(key string) (bool, error) {
	conn := r.RedisConn.Get()
	defer conn.Close()

	return redis.Bool(conn.Do("DEL", key))
}

// LikeDeletes batch delete
func (r *RedisHandler) LikeDeletes(key string) error {
	conn := r.RedisConn.Get()
	defer conn.Close()

	keys, err := redis.Strings(conn.Do("KEYS", "*"+key+"*"))
	if err != nil {
		return err
	}

	for _, key := range keys {
		_, err = r.Delete(key)
		if err != nil {
			return err
		}
	}

	return nil
}

// Incr get a key
func (r *RedisHandler) Incr(key string) (int, error) {
	conn := r.RedisConn.Get()
	defer conn.Close()

	reply, err := redis.Int(conn.Do("INCR", key))
	if err != nil {
		return -1, err
	}

	return reply, nil
}

// GetSumLikeKeys - get the sum of all likely keys value
func (r *RedisHandler) GetSumLikeKeys(key string) (int, error) {
	conn := r.RedisConn.Get()
	defer conn.Close()

	keys, err := redis.Strings(conn.Do("KEYS", "*"+key+"*"))
	if err != nil {
		return -1, err
	}

	sum := 0
	for _, key := range keys {
		val, err := r.GetInt(key)
		if err != nil {
			return -1, err
		}

		sum += val
	}

	return sum, nil
}
