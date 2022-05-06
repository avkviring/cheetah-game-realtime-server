# Cheetah Server Platform

Серверная платформа для разработки многопользовательских игр в основном с pvp матчами реального времени.

## Структура

- [Автоматическая сборка](.github/index.md)
- [GRPC](proto/)
- [Клиентские компоненты](clients/README.md)
- [Серверные компоненты](server/README.md)
- [Хостинг](hosting/README.md)
- [Публичная документация](docs/public/README.md)

## Принципы разработки

- [Тестирование](docs/private/test.md)
- [Документирование](docs/README.md)
- [Проверочные листы для разных ситуаций](docs/private/checklists/README.md)
- [Наименование commit](CONTRIBUTING.md)

## Шаблон микросервиса

Минимальный шаблон микросервиса расположен в каталоге template.

Устанавливаем шаблонизатор:

```
cargo install --force guidon-cli
```

Заполняем template.toml в корне проекта.

Запускаем шаблонизатор:

```yaml
guic tplt . .
```
