# Запуск GameServer в локальном Kubernetes кластере

Этот гайд описывает запуск одного GameServer в локальном кластере.
Гайд будет полезен для тестирования интеграции Agones SDK

[Официальный гайд](https://agones.dev/site/docs/getting-started/create-gameserver/)

1. В локальном docker должен быть образ `docker.registry.cheetah.games/cheetah-matches-relay:999.999.999`.
    Образ можно собрать по [инструкции](../gameserver-docker-build.md).
2. Загрузить docker образ в minikube
    ```bash
    minikube image load docker.registry.cheetah.games/cheetah-matches-relay:999.999.999
    ```
3. Создать GameServer
    ```bash
   cd platform
   kubectl create -f ./hosting/dev/agones-minikube-gameserver/gameserver.yaml
    ```
4. Проверить статус GameServer
    ```bash
    kubectl get gs
    kubectl describe gs
    ```

## Troubleshooting

Если GameServer не в статусе Ready, то можно найти ошибку в логах

```bash
kubectl get pods # найти имя пода
kubectl logs -f CHANGE_TO_POD_NAME -c relay # логи GameServer
kubectl logs -f CHANGE_TO_POD_NAME -c agones-gameserver-sidecar # логи Agones sidecar
````
