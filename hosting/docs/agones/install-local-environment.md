# Настройка локального окружения для работы с Agones

Этот гайд поможет настроить локальный Kubernetes кластер с настроенным Agones.
Локальный кластер полезен для разработки helm чартов и интеграция с Agones SDK в GameServer сервере.

## MacOS

1. Установить зависимости

    ```bash
    brew install --cask docker
    brew install kubernetes-cli helm minikube
    ```

2. Запустить minikube кластер ([док](https://minikube.sigs.k8s.io/docs/start/))

    ```bash
    minikube start
    ```

3. Установить Agones в Minikube кластер ([док](https://agones.dev/site/docs/installation/install-agones/helm/))

    ```bash
    helm repo add agones https://agones.dev/chart/stable
    helm repo update
    helm install agones --namespace agones-system --create-namespace agones/agones
    ```

4. Проверить инсталляцию по [доке](https://agones.dev/site/docs/installation/confirm/)
