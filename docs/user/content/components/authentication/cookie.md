# Временная регистрация

Сервер создает нового игрока и выдает код авторизации, который сохраняется на клиенте. Если пользователь переставит
клиент — код и доступ к данным сервера будет потерян.

Такой способ применяется для быстрой регистрации на начальном этапе игры, далее очень желательно привязать существующий
аккаунт к внешней системе авторизации (например, к google). После привязки код авторизации удаляется с сервера.

### Пример кода

<!- Оригинал - Assets/Scripts/DocumentationExample.cs функция CookieAuthExample ->

```csharp
var clusterConnector = new ClusterConnector("some-cluster", 5000, true);
var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
var authenticationResult = await cookieAuthenticator.LoginOrRegister();
var user = authenticationResult.User;    
```
