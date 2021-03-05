#!/bin/bash
protoc \
--proto_path=../../cerberus/proto/ \
--proto_path=. \
--plugin=protoc-gen-grpc=/usr/local/Cellar/grpc/1.33.2_3/bin/grpc_csharp_plugin \
--grpc_out=../clients/Unity/Packages/games.cheetah.unity.auth.cookie/Runtime/grpc/  \
--csharp_out=../clients/Unity/Packages/games.cheetah.unity.auth.cookie/Runtime/grpc/ cookie.proto \

protoc \
--proto_path=../../cerberus/proto/ \
--proto_path=. \
--plugin=protoc-gen-grpc=/usr/local/Cellar/grpc/1.33.2_3/bin/grpc_csharp_plugin \
--grpc_out=../clients/Unity/Packages/games.cheetah.unity.auth.android/Runtime/grpc/  \
--csharp_out=../clients/Unity/Packages/games.cheetah.unity.auth.android/Runtime/grpc/ google.proto \

