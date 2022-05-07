## Выкладка платформы в kubernetes для локальных тестов
cargo run --manifest-path ../Configurator/Cargo.toml ../example-config Platform/templates/
helm --kubeconfig /Users/kviring/Documents/.kube/stage.yaml -n $1 uninstall $1
kubectl --kubeconfig /Users/kviring/Documents/.kube/stage.yaml delete --namespace $1 --all deployments,statefulsets,services,pods,pvc
helm --kubeconfig /Users/kviring/Documents/.kube/stage.yaml -n $1 upgrade \
  --install $1 Platform \
  -f Platform/values-dev.yaml \
  --set global.grpcDomain=$1.stage.cheetah.games \
  --set global.platformImageVersion=999.999.999
