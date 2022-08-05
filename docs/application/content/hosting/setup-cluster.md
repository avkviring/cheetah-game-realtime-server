# Настройка кластера

Кластер нужен для предоставления доступа к игре внешним тестировщикам или игрокам. При этом вся разработка
осуществляется без кластера, так как платформа может запускаться локально на компьютере разработчика.

Для установки нужны минимальные знания в linux и kubernetes.

## Последовательность установки кластера

- создать kubernetes кластер в облачном провайдере - DigitalOcean, Selectel , AWS;
- склонировать репозиторий с установщиком
  ```bash 
        git clone git@github.com:cheetah-game-platform/devops.git 
  ```
- выполнить установку ([подробнее](https://github.com/cheetah-game-platform/devops#readme))
  ```bash
    ./prepare-cluster.sh DOMAIN EMAIL PROMETHEUS_PASSWORD GRAFANA_PASSWORD
  ```
