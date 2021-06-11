## Настройка Digital Ocean

### Установка Ingress контроллера

```
kubectl create namespace ingress-nginx
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm install nginx-ingress ingress-nginx/ingress-nginx --namespace=ingress-nginx --version 3.33.0 --set controller.publishService.enabled=true
```

### Доменное имя

В панели DO связать доменное имя с созданным балансировщиком

### Управление сертификатами

```
kubectl create namespace cert-manager
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm install cert-manager jetstack/cert-manager --namespace cert-manager --version v1.3.1 --set installCRDs=true   
```
### Linkerd
Установить, по-инструкции - https://linkerd.io/2.10/getting-started/

### Секреты для платформы

### Доступ к реестру

```shell
kubectl create secret docker-registry cheetahdockerregistry --docker-server=docker.registry.cheetah.games --docker-username=ci --docker-password=QPAL-OELR-PLFP-QNSD --docker-email=alex@kviring.com
```

### JWT ключи

```shell
kubectl create secret generic jwt \
--from-literal=public=LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVlZITlhLeG9VTmtvWDlobk9KcFN6NksyS0RmaQpneGFTWHUrRklwUDMycXZjRGdaUFowMXRqbkdqT3lzeVB4Um9aYU11L2Q5ckhpM3VsYmNlb1l3UytRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0t \
--from-literal=private=LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZ2NnN2RzSldTejhmNDBjRXYKQlRlR1N6QU5YR2xFenV0ZDlJSW02L2lubDBhaFJBTkNBQVJWVWMxY3JHaFEyU2hmMkdjNG1sTFBvcllvTitLRApGcEplNzRVaWsvZmFxOXdPQms5blRXMk9jYU03S3pJL0ZHaGxveTc5MzJzZUxlNlZ0eDZoakJMNQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0t
```

## Работа с Helm чартами

### Push

```
export HELM_EXPERIMENTAL_OCI=1
helm registry login -u kviring docker.registry.cheetah.games
helm chart save .  docker.registry.cheetah.games/platform:999.999.999
helm chart push  docker.registry.cheetah.games/platform:999.999.999
```