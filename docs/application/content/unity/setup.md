# Unity

Компоненты платформы подключаются к Unity посредством Package Manager.

## Настройка репозитория

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

## Демо проект

Демонстрационный проект доступен на
[GitHub](https://github.com/cheetah-games/demo-game).
