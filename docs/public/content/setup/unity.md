# Unity

Компоненты платформы подключаются к Unity посредством Package Manager. 

### Настройка репозитория
Необходимо добавить в UnityProject/Packages/manifest.json следующие строки:

```json
"scopedRegistries": [
  {
    "name": "cheetah",
    "url": "https://npm.registry.cheetah.games/",
    "scopes": [
      "games.cheetah"
      ]
  }
]
``` 

### Настройка авторизации
Для начала работы необходимо прописать полученный токен в файл .upmconfig.toml

```jsx
[npmAuth."https://npm.registry.cheetah.games"]
token = ""
alwaysAuth = true
```
[Документация на сайте Unity](https://docs.unity3d.com/Manual/upm-config.html)
