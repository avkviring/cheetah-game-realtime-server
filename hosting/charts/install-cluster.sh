#!/bin/bash
#
# Подготовка кластера, установка необходимых сервисов (кроме собственно игры)
#

# set default storage class
# kubectl patch storageclass fast.ru-2c  -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'

# install nginx
kubectl create namespace ingress-nginx
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
(
  helm upgrade \
    --install nginx-ingress ingress-nginx/ingress-nginx \
    --namespace=ingress-nginx \
    --version 4.1.0 \
    --set controller.publishService.enabled=true \
    --set "controller.tolerations[0].key=type" \
    --set "controller.tolerations[0].value=nginx" \
    --set "controller.tolerations[0].operator=Equal" \
    --set "controller.tolerations[0].effect=NoSchedule" \
    --set "controller.nodeSelector.type=nginx" \
    --set "defaultBackend.tolerations[0].key=type" \
    --set "defaultBackend.tolerations[0].value=nginx" \
    --set "defaultBackend.tolerations[0].operator=Equal" \
    --set "defaultBackend.tolerations[0].effect=NoSchedule" \
    --set "defaultBackend.nodeSelector.type=nginx" \
    --set "controller.admissionWebhooks.patch.tolerations[0].key=type" \
    --set "controller.admissionWebhooks.patch.tolerations[0].value=nginx" \
    --set "controller.admissionWebhooks.patch.tolerations[0].operator=Equal" \
    --set "controller.admissionWebhooks.patch.tolerations[0].effect=NoSchedule" \
    --set "controller.admissionWebhooks.patch.nodeSelector.type=nginx"
)

# install cert-manager
kubectl create namespace cert-manager
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm upgrade --install cert-manager jetstack/cert-manager --namespace cert-manager --version v1.5.0 --set installCRDs=true

# install system
kubectl create namespace system
helm -n system upgrade --install system System

# install agones
kubectl create namespace production
kubectl create namespace agones-system
helm upgrade --install agones agones/agones --set "gameservers.namespaces={production}" --namespace agones-system --set agones.ping.install=false --set agones.allocator.install=false

# install ydb
kubectl create ns ydb
helm install ydb-operator ydb/operator -n ydb

# install linkerd
helm repo add linkerd https://helm.linkerd.io/stable
step certificate create root.linkerd.cluster.local ca.crt ca.key --profile root-ca --no-password --insecure
step certificate create identity.linkerd.cluster.local issuer.crt issuer.key --profile intermediate-ca --not-after 8760h --no-password --insecure --ca ca.crt --ca-key ca.key
kubectl create ns linkerd

# Добавить необходимые параметры
./update-monitoring.sh

helm install linkerd2 \
  --set-file identityTrustAnchorsPEM=ca.crt \
  --set-file identity.issuer.tls.crtPEM=issuer.crt \
  --set-file identity.issuer.tls.keyPEM=issuer.key \
  linkerd/linkerd2

helm upgrade --install linkerd-viz linkerd/linkerd-viz \
  --set prometheus.enabled=false \
  --set grafana.enabled=false \
  --set prometheusUrl=http://monitoring-kube-prometheus-prometheus.monitoring:9090 \
  --set grafanaUrl=http://monitoring-grafana.monitoring:80
