# Концепции

- [Комнаты](basic/room.md)
- [Объекты](basic/object.md)
- [Клиенты](basic/client.md)
- [Каналы](basic/channel.md)

# Сервер

Сервер поставляется как docker image. Для его запуска необходимо выполнить команду:

```bash
 docker run ghcr.io/cheetah-game-platform/platform/cheetah-server:0.2.12
```

# Unity

Для получения доступа к пакетам необходимо добавить в
UnityProject/Packages/manifest.json следующие строки:

```json
"scopedRegistries": [
  {
    "name": "cheetah",
    "url": "https://npm.cheetah.games",
    "scopes": ["games.cheetah"]
  }
]
```

После настройки репозитория необходимо выбрать необходимые пакеты в Package
Manager.

## Unity пакеты

- **games.cheetah.client** - клиент для работы с сервером;
- **games.cheetah.embedded-server** - встроенный сервер, можно использовать для тестов, также его можно использовать для
  production режима;
- **games.cheetah.uds** - использование Unity Dedicated Server как одного из клиентов с неограниченными правами;

## Документация
- [Панель для просмотра команд](unity/commands-panel.md)
- [Панель для просмотра состояний](unity/dump-panel.md)
- [Интеграционные тесты](unity/integration.md)
- [Настройка логирования](unity/logger.md)
- [Эмуляция характеристик сети](unity/network.md)
- [Сериализация](unity/serialization.md)
- [Unity Dedicated Server](unity/uds.md)
- [Примеры использования API](../client/Unity/Packages/games.cheetah.client/Tests/Server)
