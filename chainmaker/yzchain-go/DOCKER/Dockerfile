FROM golang:1.16.6 as builder
ENV GOPROXY=https://goproxy.cn,direct
ENV GOPRIVATE=yzchain.org
COPY . /yzchain-go
RUN cd /yzchain-go && go mod tidy && make && make cmc

# the second stage
FROM ubuntu:20.04
RUN rm /bin/sh && ln -s /bin/bash /bin/sh
RUN sed -i "s@http://.*archive.ubuntu.com@http://mirrors.tuna.tsinghua.edu.cn@g" /etc/apt/sources.list && \
    sed -i "s@http://.*security.ubuntu.com@http://mirrors.tuna.tsinghua.edu.cn@g" /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y vim net-tools tree gcc g++ p7zip-full

ENV TZ "Asia/Shanghai"
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y tzdata && \
    echo $TZ > /etc/timezone && \
    ln -fs /usr/share/zoneinfo/$TZ /etc/localtime && \
    dpkg-reconfigure tzdata -f noninteractive

COPY --from=builder /yzchain-go/main/libwasmer_runtime_c_api.so /usr/lib/libwasmer.so
COPY --from=builder /yzchain-go/main/prebuilt/linux/wxdec /usr/bin
COPY --from=builder /yzchain-go/bin/yzchain /yzchain-go/bin/
COPY --from=builder /yzchain-go/bin/cmc /yzchain-go/bin/
COPY --from=builder /yzchain-go/config /yzchain-go/config/
RUN mkdir -p /yzchain-go/log/
RUN chmod 755 /usr/bin/wxdec

WORKDIR /yzchain-go/bin
