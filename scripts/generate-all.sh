set -e

cd ../

echo "=== Generate Unity GRPC "
source scripts/generators/unity.sh
generate_unity_grpc_files proto/service.proto client/Unity/Packages/games.cheetah.realtime.grpc/Runtime/

# мета файлы создаем только для linux - так как
# именно он используется для вывода релиза
if [[ ($OSTYPE == 'linux'*) ]]; then
  generate_meta_files client/Unity/Packages/games.cheetah.realtime.grpc/Runtime/
fi
