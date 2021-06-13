# Хостинг

Платформа запускается в Kubernetes кластере. Для управления конфигурацией используется Helm.

## Настройка нового кластера в Digital Ocean

### Установка Ingress контроллера

```
kubectl create namespace ingress-nginx
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm install nginx-ingress ingress-nginx/ingress-nginx --namespace=ingress-nginx --version 3.33.0 --set controller.publishService.enabled=true
```

### Управление сертификатами

```
kubectl create namespace cert-manager
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm install cert-manager jetstack/cert-manager --namespace cert-manager --version v1.3.1 --set installCRDs=true   

cd hosting/charts/System
kubectl create namespace system
helm upgrade --namespace=system --install system .
```

### Linkerd

Установить, по-инструкции - https://linkerd.io/2.10/getting-started/

### Доменное имя

В панели DigitalOcean связать доменное имя с созданным балансировщиком.

## Настройка глобальных секретов

### Доступ к реестру

```shell
kubectl create secret docker-registry cheetahdockerregistry \ 
--docker-server=docker.registry.cheetah.games  \
--docker-username=***  \
--docker-password=***  \
--docker-email=***
```

### JWT ключи

```shell
kubectl create secret generic jwt \
--from-literal=public=***
--from-literal=private=***
```

# Справочная информация

## Работа с Helm чартами

### Push

```
export HELM_EXPERIMENTAL_OCI=1
helm registry login -u kviring docker.registry.cheetah.games
helm chart save .  docker.registry.cheetah.games/platform:999.999.999
helm chart push  docker.registry.cheetah.games/platform:999.999.999
```