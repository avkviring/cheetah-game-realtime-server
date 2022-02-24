using Cheetah.Accounts.Cookie;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Platform;

/// <summary>
/// Примеры для документации, если в процессе рефакторинга в этом классе есть изменения - то их необходимо перенести в документацию
/// </summary>
public class DocumentationExample
{
    // http://127.0.0.1:8000/components/matchmaking/
    // docs/public/content/components/matchmaking/index.md
    public async void MatchmakingExample()
    {
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
    }
}