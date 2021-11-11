# Matchmaking

Выбор наиболее подходящего игрового сервера для конретного игрока.

### Настройка среды разработки

Установить пакет **games.cheetah.matches.matchmaking** в Package Manager.

### Пример кода

```csharp
// создаем нового пользователя
var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
cookieAuthenticator.RemoveLocalCookie();
var loginOrRegister = await cookieAuthenticator.LoginOrRegister();

// сообщаем mm о желании попасть в битву
var player = loginOrRegister.Player;
var scheduler = new MatchmakingScheduler(player);
var ticket = await scheduler.Schedule("gubaha", UserGroup);

// вход в битву
var userPrivateKey = new CheetahBuffer(ticket.PrivateKey.ToByteArray());
CheetahClient.CreateClient(ticket.RelayGameHost + ":" + ticket.RelayGamePort, (ushort)ticket.UserId, ticket.RoomId,
    ref userPrivateKey, 0, out clientId);
CheetahClient.SetCurrentClient(clientId);           
```
