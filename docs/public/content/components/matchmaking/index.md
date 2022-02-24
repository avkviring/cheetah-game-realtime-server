# Matchmaking

Выбор наиболее подходящего игрового сервера для конретного игрока.

### Настройка среды разработки

Установить пакет **games.cheetah.matches.matchmaking** в Package Manager.

### Алгоритм матчинга

В настоящее время реализована только заглушка - она закидывает всех пользователей в одну и ту же комнату и на один и тот
же relay сервер. Для разных шаблонов создаются разные комнаты. В случае падения relay сервера необходимо перезапустить
matchmaking и registry.

### Пример кода

<!---
 Оригинал clients/Unity/Assets/Scripts/DocumentationExample.cs функция MatchmakingExample
-->

```csharp           
// соединяемся с кластером платформы
var clusterConnector = new ClusterConnector("some-cluster", 5000, true);

// создаем пользователя - в данном примере 
// используется авторизация по сгенеренному коду
// использовать только в тестах
var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
cookieAuthenticator.RemoveLocalCookie();
var authenticationResult = await cookieAuthenticator.LoginOrRegister();
var player = authenticationResult.User;

// запрос на подбор битвы
var scheduler = new MatchmakingScheduler(player);
var ticket = await scheduler.Schedule("roomTemplate", 777);

// соединение с боевым сервером
var client = new CheetahClient(
    ticket.RelayGameHost,
    ticket.RelayGamePort,
    ticket.UserId,
    ticket.RoomId,
    ticket.PrivateKey.ToByteArray(),
    new CodecRegistryBuilder().Build());           
```
