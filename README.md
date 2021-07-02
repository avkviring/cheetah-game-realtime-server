# Cheetah Server Platform

Серверная платформа для создания многопользовательских игр.

## Структура проекта

- clients - клиентская часть
- [docs](docs/README.md) - публичная документация проекта
- [dev](dev/README.md) - утилиты/шаблоны/etc для разработчика
- [hosting](hosting/README.md) - настройка хостинга, helm чарты
- proto - grpc proto файлы
- server - серверная часть

## Документация

- [Правила документирования](docs/README.md)
- [CI система](.github/index.md)
- [Инструменты разработки](dev/README.md)

### Сервисы

- [Авторизация](server/authentication/README.md)
- [JWT токены](server/cerberus/README.md)
- [Матчи реального времени](server/matches/README.md)
  - [Сервер для обмена данными между пользователями](server/match/Relay/README.md)
  - Matchmaking для тестирования
  - MatchRegistry для тестирования

### Создание нового компонента

#### Список мест в которые необходимо добавить информацию о новом компоненте

##### Github Action

- .github/workflows/test.helm.yml
    - добавить в список для построения docker образа
    - добавить версию в параметры helm
- .github/workflows/test.server.yml
- .github/workflows/release.components.yml

##### Хостинг

- зависимость в hosting/charts/Platform/Chart.yaml
- тест запущенного компонента hosting/E2ETest

 

