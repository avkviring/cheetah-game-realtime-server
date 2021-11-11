# Временная регистрация

Сервер создает нового игрока и выдает код авторизации, который сохраняется на клиенте. 
Если пользователь переставит клиент — код и доступ к данным сервера будет потерян.

Такой способ применяется для быстрой регистрации на начальном этапе игры, далее
очень желательно привязать существующий аккаунт к внешней системе авторизации (например, к google).
После привязки код авторизации удаляется с сервера.

### Настройка среды разработки
Установить пакет **games.cheetah.auth.cookie** в Package Manager.

### Пример кода

```csharp
    var connector = new ClusterConnector("127.0.0.1", 7777, false);
    var cookieAuthenticator = new CookieAuthenticator(connector);
    var result = await cookieAuthenticator.LoginOrRegister();
    var player = result.Player;    
```
