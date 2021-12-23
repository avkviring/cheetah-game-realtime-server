## Настройка нового кластера

Тестировалось только на DigitalOcean.

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

Необходимо в gameservers.namespaces указать все namespace,в которых возможен запуск боевых серверов.

```
kubectl create namespace agones-system
helm install agones agones/agones --set "gameservers.namespaces={...}" --namespace agones-system
```

### Доменное имя

В панели DigitalOcean связать доменное имя с созданным балансировщиком.

### Настройка firewall

Открыть UDP в DigitalOcean на всех нодах.