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

#### Авторизация

Преобразование внешней авторизации в JWT токены платформы.

- Клиент - clients/Unity/Packages/games.cheetah.cerberus
- [Сервер](server/authentication/README.md)

#### JWT токены

Создание и обновление JWT токенов для авторизации внутри платформы.

- Клиент
    - clients/Unity/Packages/games.cheetah.authentication
    - clients/Unity/Packages/games.cheetah.authentication.android
    - clients/Unity/Packages/games.cheetah.authentication.cookie
- [Сервер](server/cerberus/README.md)

#### Битвы реального времени

- Клиент - clients/Unity/Packages/games.cheetah.relay
- [Сервер](server/relay/README.md)

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

 

