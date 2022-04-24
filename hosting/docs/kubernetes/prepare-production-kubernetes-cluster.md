## Настройка нового кластера

- На основе docs/adr/hosting/0001-production-kubernetes-cluster.md создаем необходимый набор нод
    - Для relay нод необходим внешний Ip адрес
- Выполняем скрипт hosting/charts/prepare-production-cluster.sh
    - Привязываем доменное имя *... к ip адресу nginx балансировщика
- Выполняем hosting/charts/install-monitoring.sh
- Выполняем hosting/charts/install-system.sh
- Настраиваем namespace production hosting/docs/kubernetes/prepare-namespace.md
- Выполняем hosting/charts/install-production.sh

Платформа инсталлируется в production namespace.
