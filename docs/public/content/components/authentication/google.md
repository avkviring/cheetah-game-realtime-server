# Google

Клиент получает google token и передает на сервер, который проверяет его и формирует JWT токены для доступа к сервисам.

### Настройка среды разработки

Установить пакет **games.cheetah.auth.android** в Package Manager.

Получить из google console - id приложения и WebId клиента. 
ID приложения прописать в настройки Unity проекта - Cheetah/Google PlayGames.

### Пример кода

```csharp
#if UNITY_ANDROID
    var connector = new ClusterConnector("127.0.0.1", 7777, false);
    string androidWebClientId = "some_code.apps.googleusercontent.com";
    var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
    var result = await androidAuthenticator.LoginOrRegister(connector);
    var player = result.Player;
#endif    
```
