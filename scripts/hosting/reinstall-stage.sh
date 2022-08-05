# Выкладка платформы в kubernetes для локальных тестов
# Внимание - удаляет текущий релиз со всеми сохраненным данными

helm cheetah-config-creator ../example-config Platform/templates/
helm -n $1 uninstall $1
kubectl delete --namespace $1 --all database,storage,deployments,statefulsets,services,pods,pvc
helm -n $1 upgrade --install $1 Platform \
  -f Platform/values-dev.yaml \
  --set global.grpcDomain=$1.stage.cheetah.games \
  --set global.platformImageVersion=$2 \
  --wait-for-jobs \
  --wait \
  --timeout 10m \
