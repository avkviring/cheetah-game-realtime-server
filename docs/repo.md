# Репозитарии проекта

## Unity Package

https://registry.cheetah.games/

Добавить в UnityProject/Packages/manifest.json конфигурацию репозитория

```json
"scopedRegistries": [
  {
    "name": "cheetah",
    "url": "https://registry.cheetah.games/",
    "scopes": [
      "games.cheetah.unity"
      ]
  }
]
``` 