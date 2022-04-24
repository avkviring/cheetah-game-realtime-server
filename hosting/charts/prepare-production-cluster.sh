#!/bin/bash

#
# Подготовка production кластера, установка необходимых сервисов (кроме собственно игры)
#
KUBE_CONFIG=/Users/kviring/Documents/.kube/syncario-prod.yaml

# set default storage class
# kubectl patch storageclass fast.ru-2c  -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'

# install nginx
kubectl --kubeconfig=$KUBE_CONFIG create namespace ingress-nginx
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
(helm --kubeconfig=$KUBE_CONFIG upgrade \
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
kubectl --kubeconfig=$KUBE_CONFIG create namespace cert-manager
helm repo add jetstack https://charts.jetstack.io
helm repo update
helm --kubeconfig=$KUBE_CONFIG upgrade --install cert-manager jetstack/cert-manager --namespace cert-manager --version v1.5.0 --set installCRDs=true


# install agones
kubectl --kubeconfig=$KUBE_CONFIG create namespace production
kubectl --kubeconfig=$KUBE_CONFIG create namespace agones-system
helm --kubeconfig=$KUBE_CONFIG upgrade --install agones agones/agones --set "gameservers.namespaces={production}" --namespace agones-system --set agones.ping.install=false --set agones.allocator.install=false


