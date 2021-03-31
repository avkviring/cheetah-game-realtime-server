# Google

Клиент получает google token и передает на сервер, который проверяет его и формирует JWT токены для доступа к сервисам.

### Настройка среды разработки

Установить пакет **games.cheetah.auth.android** в Package Manager.

Получить из google console - id приложения и WebId клиента. 
ID приложения прописать в настройки Unity проекта - Cheetah/Google PlayGames.

### Пример кода

```csharp
#if UNITY_ANDROID
    var connector = new Connector("192.168.212.97:7777");
    string androidWebClientId = "some_code.apps.googleusercontent.com";
    var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
    var result = await androidAuthenticator.LoginOrRegister(connector);
    var player = result.Player;
#endif    
```
