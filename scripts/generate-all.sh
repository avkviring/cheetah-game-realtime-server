set -e

cd ../

echo "=== Generate Unity GRPC "
source scripts/generators/unity.sh

generate_unity_grpc_files proto/matches.realtime.admin.proto client/Unity/Packages/games.cheetah.grpc.admin/Editor/
generate_unity_grpc_files proto/matches.realtime.shared.proto client/Unity/Packages/games.cheetah.grpc.shared/Runtime/
generate_unity_grpc_files proto/matches.realtime.internal.proto client/Unity/Packages/games.cheetah.grpc.internal/Runtime/

# мета файлы создаем только для linux - так как
# именно он используется для вывода релиза
if [[ ($OSTYPE == 'linux'*) ]]; then
  generate_meta_files client/Unity/Packages/games.cheetah.grpc.admin/Editor/
  generate_meta_files client/Unity/Packages/games.cheetah.grpc.shared/Runtime/
  generate_meta_files client/Unity/Packages/games.cheetah.grpc.internal/Runtime/
fi
