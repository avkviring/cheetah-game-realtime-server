using System.Threading.Tasks;
using Cheetah.User.Accounts.GRPC;
using Cheetah.Platform;
using Grpc.Core;
using JetBrains.Annotations;
using UnityEngine;

namespace Cheetah.User.Accounts
{
    public class StoredTokenUserAuthenticator
    {
        private string tokenKey;

        public StoredTokenUserAuthenticator(string prefix = "default")
        {
            tokenKey = "stored_player_authenticator_" + prefix;
        }

        /// <summary>
        /// Вход по сохранненым ключам доступа
        /// </summary>
        /// <returns>null - нет сохраненных ключей или они не действительны, требуется внешняя авторизация</returns>
        [ItemCanBeNull]
        public async Task<User> Login(ClusterConnector clusterConnector)
        {
            var token = PlayerPrefs.GetString(tokenKey, "");
            if (token == "")
            {
                return null;
            }

            return await clusterConnector.DoRequest(async channel =>
            {
                var client = new Tokens.TokensClient(channel);
                var request = new RefreshTokenRequest { Token = token };
                try
                {
                    var result = await client.refreshAsync(request);
                    DoStore(result.Refresh);
                    return new User(clusterConnector, result.Session, result.Refresh);
                }
                catch (RpcException e)
                {
                    // токены не валидны - требуется внешняя авторизация
                    if (e.StatusCode == StatusCode.Unauthenticated)
                    {
                        return null;
                    }

                    throw;
                }
            });
        }

        /// <summary>
        /// Сохранить токен для последующего входа без внешней авторизации
        /// </summary>
        /// <param name="user"></param>
        public void Store(User user)
        {
            DoStore(user.RefreshToken);
        }


        private void DoStore(string refreshToken)
        {
            PlayerPrefs.SetString(tokenKey, refreshToken);
            PlayerPrefs.Save();
        }


        /// <summary>
        /// Удалить токен доступа, после удаления для входа потребуется внешнияя авторизация
        /// </summary>
        public void RemoveToken()
        {
            PlayerPrefs.DeleteKey(tokenKey);
            PlayerPrefs.Save();
        }
    }
}