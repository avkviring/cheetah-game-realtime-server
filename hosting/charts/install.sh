## Выкладка платформы в kubernetes для локальных тестов
tar -czf Platform/charts/matches-factory/rooms-configuration.tgz -C ../../server/matches/Factory/example/ .
helm -n $1 uninstall $1
kubectl delete --namespace $1 --all deployments,statefulsets,services,pods,pvc,pv
helm -n $1 upgrade \
--install $1 Platform \
-f Platform/values-dev.yaml \
--set global.grpcDomain=$1.cluster.dev.cheetah.games \
--set global.platformImageVersion=999.999.999

