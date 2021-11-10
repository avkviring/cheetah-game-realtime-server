# Хостинг

Платформа запускается в Kubernetes кластере. Для управления конфигурацией используется Helm.

- charts - helm чарты платформы

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
helm install cert-manager jetstack/cert-manager --namespace cert-manager --version v1.5.0 --set installCRDs=true   

cd hosting/charts/System
kubectl create namespace system
helm upgrade --namespace=system --install system .
```

### Установка agones
```
kubectl create namespace agones-system
helm install agones agones/agones --set "gameservers.namespaces={kviring,zakharovvi,dependabot}" --namespace agones-system
```

### Linkerd

Установить, по-инструкции - https://linkerd.io/2.10/getting-started/

### Доменное имя

В панели DigitalOcean связать доменное имя с созданным балансировщиком.

### Настройка firewall
Открыть UDP в DigitalOcean на всех нодах.

# Настройка namespace для запуска платформы

В каждом namespace для запуска платформы.

### Доступ к реестру

```shell
kubectl create secret docker-registry cheetahdockerregistry \
--namespace *** \ 
--docker-server=docker.registry.cheetah.games  \
--docker-username=***  \
--docker-password=***  \
--docker-email=***
```

### JWT ключи

```shell
openssl ecparam -name prime256v1 -genkey -out private.pem
openssl pkcs8  -topk8 -nocrypt -in private.pem -out private-pkcs8.pem
openssl ec -in private.pem -pubout -out public.pem
kubectl create secret generic jwt \
--namespace *** \
--from-file=public=public.pem \
--from-file=private=private-pkcs8.pem
```

# Справочная информация

## Работа с Helm чартами

### Push

```
export HELM_EXPERIMENTAL_OCI=1
helm registry login -u kviring docker.registry.cheetah.games
helm chart save .  docker.registry.cheetah.games/platform:999.999.999
helm chart push  docker.registry.cheetah.games/platform:999.999.999

      - name: Deploy Helm Chart
        run: |
          export HELM_EXPERIMENTAL_OCI=1
          helm registry login -u ${{ secrets.DOCKER_REGISTRY_USER }} -p ${{ secrets.DOCKER_REGISTRY_PASSWORD }} docker.registry.cheetah.games
          cd server/${{ matrix.component }}/Chart/
          sed -i.bak 's/999.999.999/${{  github.sha }}/' Chart.yaml
          helm chart save . docker.registry.cheetah.games/${{ matrix.component }}:${{ github.sha }}
          helm chart push  docker.registry.cheetah.games/${{ matrix.component }}:${{ github.sha }}
```