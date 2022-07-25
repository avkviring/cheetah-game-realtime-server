# Google

Клиент получает google token и передает на сервер, который проверяет его и формирует токены для доступа к сервисам.

### Настройка среды разработки

Получить из google console - id приложения и WebId клиента. ID приложения прописать в настройки Unity проекта -
Cheetah/Google PlayGames.

### Пример кода

<!- Оригинал - Assets/Scripts/DocumentationExample.cs функция GoogleAuthExample ->
```csharp
#if UNITY_ANDROID
    var clusterConnector = new ClusterConnector("some-cluster", 5000, true);
    var androidWebClientId = "some_code.apps.googleusercontent.com";
    var androidAuthenticator = new GoogleAuthenticator(androidWebClientId);
    var result = await androidAuthenticator.LoginOrRegister(clusterConnector);
    var user = result.User;
#endif    
```
