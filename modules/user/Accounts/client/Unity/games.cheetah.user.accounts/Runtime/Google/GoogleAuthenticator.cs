#if UNITY_ANDROID
using System.Threading.Tasks;
using Cheetah.Accounts.GRPC;
using Cheetah.Platform;
using GooglePlayGames;
using GooglePlayGames.BasicApi;
using Grpc.Core;
using UnityEngine;

namespace Cheetah.Accounts.Google
{
    public class GoogleAuthenticator
    {
        private string webClientId;

        public GoogleAuthenticator(string webClientId)
        {
            this.webClientId = webClientId;
        }

        /// <summary>
        /// Авторизация игрока с помощью google android.
        /// Если игрока не сервере нет - он будет создан.
        /// </summary>
        /// <param name="connector"></param>
        /// <exception cref="GoogleAuthenticateException">Ошибка при использовании android авторизации</exception>
        /// <exception cref="RpcException">Ошибка сетевого взаимодействия с платформой</exception>
        /// <returns>Результат авторизации</returns>
        public async Task<GoogleAuthenticationResult> LoginOrRegister(ClusterConnector connector)
        {
            PlayGamesClientConfiguration config =
                new PlayGamesClientConfiguration.Builder(webClientId)
                    .RequestIdToken()
                    .Build();
            PlayGamesPlatform.DebugLogEnabled = true;
            PlayGamesPlatform.InitializeInstance(config);
            PlayGamesPlatform.Activate();


            var task = new TaskCompletionSource<string>();

            PlayGamesPlatform.Instance.Authenticate(SignInInteractivity.CanPromptAlways, (result) =>
            {
                if (result == SignInStatus.Success)
                {
                    task.SetResult(PlayGamesPlatform.Instance.GetIdToken());
                }
                else
                {
                    task.SetException(new GoogleAuthenticateException(result));
                }
            });


            var googleToken = await task.Task;
            return await connector.DoRequest(async channel =>
            {
                var client = new GRPC.Google.GoogleClient(channel);
                var request = new RegisterOrLoginRequest { DeviceId = SystemInfo.deviceUniqueIdentifier, GoogleToken = googleToken };
                var result = await client.RegisterOrLoginAsync(request);
                return new GoogleAuthenticationResult(result.RegisteredPlayer, new User(connector, result.Tokens.Session, result.Tokens.Refresh),
                    PlayGamesPlatform.Instance.GetUserDisplayName());
            });
        }
    }
}
#endif