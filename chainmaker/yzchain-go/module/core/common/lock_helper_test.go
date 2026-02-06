package common

import (
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

/*
 * test unit ReentrantLock func
 */
func Test_ReentrantLock(t *testing.T) {
	lock := &ReentrantLocks{
		ReentrantLocks: make(map[string]interface{}),
		Mu:             sync.Mutex{},
	}

	for i := 0; i < 3; i++ {
		go func(i int) {
			if lock.Lock("") {
				require.False(t, lock.Lock(""))
				defer lock.Unlock("")
				fmt.Printf("%d get lock \n", i)
				time.Sleep(2 * time.Second)
			}
		}(i)
	}

	for i := 0; i < 3; i++ {
		go func(i int) {
			for {
				if lock.Lock("") {
					defer lock.Unlock("")
					fmt.Printf("finally %d get lock \n", i)
					break
				}
			}
		}(i)
	}

	time.Sleep(5 * time.Second)
}

/*
 * test unit ReentrantLocks func
 */
func Test_ReentrantLocks(t *testing.T) {
	locks := &ReentrantLocks{
		ReentrantLocks: make(map[string]interface{}),
		Mu:             sync.Mutex{},
	}
	for i := 0; i < 3; i++ {
		go func(i int) {
			if locks.Lock("1") {
				require.False(t, locks.Lock("1"))
				defer locks.Unlock("1")
				fmt.Printf("%d get lock", i)
				time.Sleep(2 * time.Second)
			}
		}(i)
	}

	for i := 0; i < 3; i++ {
		go func(i int) {
			for {
				if locks.Lock("2") {
					defer locks.Unlock("2")
					fmt.Printf("finally %d get lock \n", i)
					time.Sleep(1 * time.Second)
					break
				}
			}
		}(i)
	}
	time.Sleep(5 * time.Second)

}

/*
 * test unit reentrantLock func
 */
type reentrantLock struct {
	reentrantLock *int32
}

/*
 * test unit lock func
 */
func (l *reentrantLock) lock(key string) bool {
	return atomic.CompareAndSwapInt32(l.reentrantLock, 0, 1)
}

/*
 * test unit unlock func
 */
func (l *reentrantLock) unlock(key string) bool {
	return atomic.CompareAndSwapInt32(l.reentrantLock, 1, 0)
}

/*
 * test unit ReentrantLocks Unlock func
 */
func TestReentrantLocks_Unlock(t *testing.T) {
	type fields struct {
		ReentrantLocks map[string]interface{}
	}
	type args struct {
		key string
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name: "test0",
			fields: fields{
				ReentrantLocks: nil,
			},
			args: args{
				key: LOCKED,
			},
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			l := &ReentrantLocks{
				ReentrantLocks: tt.fields.ReentrantLocks,
			}
			if got := l.Unlock(tt.args.key); got != tt.want {
				t.Errorf("Unlock() = %v, want %v", got, tt.want)
			}
		})
	}
}

/*
 * test unit ReentrantLocks Lock func
 */
func TestReentrantLocks_Lock(t *testing.T) {
	type fields struct {
		ReentrantLocks map[string]interface{}
	}
	type args struct {
		key string
	}
	tests := []struct {
		name   string
		fields fields
		args   args
		want   bool
	}{
		{
			name: "test0",
			fields: fields{
				ReentrantLocks: map[string]interface{}{
					"test": LOCKED,
				},
			},
			args: args{
				key: LOCKED,
			},
			want: true,
		},
		{
			name: "test1",
			fields: fields{
				ReentrantLocks: map[string]interface{}{
					"test": "test",
				},
			},
			args: args{
				key: "test",
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			l := &ReentrantLocks{
				ReentrantLocks: tt.fields.ReentrantLocks,
			}
			if got := l.Lock(tt.args.key); got != tt.want {
				t.Errorf("Lock() = %v, want %v", got, tt.want)
			}
		})
	}
}
