# Прокидывание порта для доступа к web консоли viz (данные network mesh linkerd)
kubectl -n linkerd-viz port-forward service/web 8084:8084