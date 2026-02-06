## Introduction

```bash
# tree -L 1
.
├── README.md
├── gmssl # this is from gmssl open source project, and do some modification
├── sm2  # a sm2 wrapper of gmssl sm2 algorithm
└── sm3 # a sm3 wrapper of gmssl sm2 algorithm
```

## changelog
There are some optimization in sub-package gmssl, such as:
* add algorithm constant definition
* refine condition compile `+build cgo`
* refine GeneratePrivateKey implementation, wrong use of `defer` etc.
* cgo `LDFLAGS` and `CFLAGS`
