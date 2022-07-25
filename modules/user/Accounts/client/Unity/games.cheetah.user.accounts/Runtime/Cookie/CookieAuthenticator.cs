using System.Threading.Tasks;
using Cheetah.User.Accounts.GRPC;
using Cheetah.Platform;
using UnityEngine;

namespace Cheetah.User.Accounts.Cookie
{
    /// <summary>
    /// Авторизация по коду, который хранится на клиенте
    /// При удалении игры доступ к данным игрока будет потерян.
    /// Используется в тестовых сборках или для временной регистрации, если нет возможности использовать внешние авторизационные системы
    /// </summary>
    public class CookieAuthenticator
    {
        private ClusterConnector clusterConnector;
        private string key;

        /// <summary>
        ///
        /// </summary>
        /// <param name="clusterConnector"></param>
        /// <param name="prefix">Необходимо использовать если требуется зарегистрировать несколько пользователей, например для тестирования</param>
        public CookieAuthenticator(ClusterConnector clusterConnector, string prefix = "default")
        {
            this.clusterConnector = clusterConnector;
            key = "cookie_player_authenticator_" + prefix;
        }

        /// <summary>
        /// Создать игрока или войти под существующем
        /// </summary>
        /// <exception cref="CookieNotFoundException">
        ///     На сервере нет игрока для данного cookie, например, при потере базы данных
        /// </exception>
        /// <returns></returns>
        public async Task<CookieAuthenticationResult> LoginOrRegister()
        {
            return await clusterConnector.DoRequest(async channel =>
            {
                var client = new GRPC.Cookie.CookieClient(channel);
                var cookie = PlayerPrefs.GetString(key, "");
                if (cookie == "")
                {
                    var request = new RegistryRequest { DeviceId = SystemInfo.deviceUniqueIdentifier };
                    var result = await client.RegisterAsync(request);
                    PlayerPrefs.SetString(key, result.Cookie);
                    return new CookieAuthenticationResult(true, new User(clusterConnector, result.Tokens.Session, result.Tokens.Refresh));
                }
                else
                {
                    var request = new LoginRequest { Cookie = cookie, DeviceId = SystemInfo.deviceUniqueIdentifier };
                    var result = await client.LoginAsync(request);
                    if (result.Tokens == null)
                    {
                        throw new CookieNotFoundException();
                    }

                    return new CookieAuthenticationResult(false, new User(clusterConnector, result.Tokens.Session, result.Tokens.Refresh));
                }
            });
        }


        public void RemoveLocalCookie()
        {
            PlayerPrefs.SetString(key, null);
        }
    }
}