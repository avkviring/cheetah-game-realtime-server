using Cheetah.Accounts;
using Cheetah.Accounts.Cookie;
#if UNITY_ANDROID
using Cheetah.Accounts.Google;
#endif
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Platform;

/// <summary>
/// Примеры для документации, если в процессе рефакторинга в этом классе есть изменения - то их необходимо перенести в документацию
/// </summary>
public class DocumentationExample
{
    // http://127.0.0.1:8000/components/authentication/
    // docs/public/content/components/authentication/index.md
    public async void AuthExample()
    {
        // соединяемся с кластером платформы
        var clusterConnector = new ClusterConnector("some-cluster", 5000, true);

        // попытка входа с сохраненными токенами
        var storedTokenUserAuthenticator = new StoredTokenUserAuthenticator();
        var user = await storedTokenUserAuthenticator.Login(clusterConnector);
        if (user == null)
        {
            // если сохраненных токенов нет
            // то запускаем внешнию авторизацию, например Cookie
            var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
            cookieAuthenticator.RemoveLocalCookie();
            var authenticationResult = await cookieAuthenticator.LoginOrRegister();
            user = authenticationResult.User;

            // сохраняем токены для использования после перезапуска приложения
            storedTokenUserAuthenticator.Store(user);
        }
    }

    // http://127.0.0.1:8000/components/authentication/cookie/
    // docs/public/content/components/authentication/cookie.md
    public async void CookieAuthExample()
    {
        var clusterConnector = new ClusterConnector("some-cluster", 5000, true);
        var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
        var authenticationResult = await cookieAuthenticator.LoginOrRegister();
        var user = authenticationResult.User;
    }

    // http://127.0.0.1:8000/components/authentication/google/
    // docs/public/content/components/authentication/google.md
    public async void GoogleAuthExample()
    {
#if UNITY_ANDROID
        var clusterConnector = new ClusterConnector("some-cluster", 5000, true);
        var androidWebClientId = "some_code.apps.googleusercontent.com";
        var androidAuthenticator = new GoogleAuthenticator(androidWebClientId);
        var result = await androidAuthenticator.LoginOrRegister(clusterConnector);
        var user = result.User;
#endif
    }

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