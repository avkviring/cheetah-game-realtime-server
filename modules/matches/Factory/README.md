# Фабрика для создания комнат

Представляет grpc сервис создания комнат в relay сервере на основе имени шаблона.
Используется из Matchmaking сервиса.
Необходимый relay сервер определяется с помощью Registry сервиса.

## Особенности

- не перечитывает шаблоны комнат - необходим рестарт сервиса в случае изменении конфигурации,
  ответственность на перезагрузку возложена на kubernetes

## Формат конфигурации
Пример представлен в каталоге examples.






