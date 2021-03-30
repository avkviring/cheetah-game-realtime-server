# Google

Клиент получает google token и передает на сервер, который проверяет его и формирует JWT токены для доступа к сервисам.

### Настройка среды разработки

Установить пакет **games.cheetah.auth.android** в Package Manager.

Получить из google console - id приложения и WebId клиента. Прописать их в настройки проекта.

### Пример кода для авторизации

```csharp
#if UNITY_ANDROID
    string androidWebClientId = "some_code.apps.googleusercontent.com";
    var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
    var result = await androidAuthenticator.LoginOrRegister(connector);
    player = result.Player;
#endif    
```
