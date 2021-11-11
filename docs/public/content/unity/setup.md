# Unity

Компоненты платформы подключаются к Unity посредством Package Manager.

### Настройка репозитория

Необходимо добавить в UnityProject/Packages/manifest.json следующие строки:

```json
"scopedRegistries": [
{
"name": "cheetah",
"url": "https://npm.registry.cheetah.games/",
"scopes": ["games.cheetah"]
}
]
``` 

После настройки репозитория необходимо выбрать необходимые пакеты в Package Manager.

### Демо проект

https://github.com/cheetah-games/demo-game
