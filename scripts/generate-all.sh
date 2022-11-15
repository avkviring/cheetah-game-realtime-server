set -e

cd ../

echo "=== Generate Unity GRPC "
scripts/generators/grpc/unity.sh

echo "=== Generate Csharp GRPC "
scripts/generators/grpc/csharp.sh
