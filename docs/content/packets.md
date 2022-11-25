## Unity
Компоненты платформы подключаются к Unity посредством Package Manager.

### Настройка репозитория

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

### Unity пакеты
- **games.cheetah.matches.realtime** - клиент для работы с сервером;
- **games.cheetah.matches.realtime.embedded-server** - встроенный сервер, можно использовать для тестов, также 
  его можно использовать для production режима;
- **games.cheetah.matches.realtime.doa** - дополнительное API для работы с сервером;
- **games.cheetah.matches.realtime.uds** - использование Unity Dedicated Server как одного из клиентов с 
  неограниченными правами;

## Nuget пакеты
- **GamesCheetah.RealtimeEmbeddedServer** - встроенный сервер, можно использовать для тестов, также
  его можно использовать для production режима;
