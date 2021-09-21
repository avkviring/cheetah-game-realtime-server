
tar -czf Platform/charts/matches-factory/rooms-configuration.tgz -C ../../server/matches/Factory/example/ .
helm -n kviring uninstall kviring
kubectl delete --namespace kviring --all deployments,statefulsets,services,pods,pvc,pv
helm -n kviring upgrade --install kviring Platform -f Platform/values-dev.yaml
