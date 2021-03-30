using Cheetah.Platform.Authenticator;
using UnityEngine;
using UnityEngine.UI;
#if UNITY_ANDROID
using Cheetah.Authenticator;
using Cheetah.Platform;
using Grpc.Core;

#endif

namespace Example.Auth
{
    public class AuthenticatorComponent : MonoBehaviour
    {
        private const string androidWebClientId = "663521173650-gkgrl7aouifjag0j5do14pul1hdqvosm.apps.googleusercontent.com";

        [SerializeField] private Text resultText;
        [SerializeField] private Button loginButton;
        private Connector connector = new Connector("192.168.212.97:7777");

        private void Start()
        {
            loginButton.onClick.AddListener(OnLogin);
        }

        private async void OnLogin()
        {
#if UNITY_ANDROID
            try
            {
                // вначале используем сохраненый токен для авторизации
                // если такой токен есть - то нам не потребуется повторный вызов
                // внешней авторизации
                var storedAuthenticator = new StoredPlayerAuthenticator();
                var player = await storedAuthenticator.Login(connector);
                if (player != null)
                {
                    resultText.text = "Login with StoredPlayerAuthenticator";
                }
                else
                {
                    // сохраненного токена нет или он не валиден
                    // необходима внешняя авторизация
                    try
                    {
                        var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
                        var result = await androidAuthenticator.LoginOrRegister(connector);
                        player = result.Player;
                        storedAuthenticator.StoreToken(player);
                        resultText.text = "Login with AndroidPlayerAuthenticator";
                    }
                    catch (AndroidAuthenticateException e)
                    {
                        resultText.text = "Android API Error";
                        Debug.LogError(e.Message);
                    }
                }
            }
            catch (RpcException e)
            {
                resultText.text = "RPC Exception";
                Debug.LogError(e.Message);
            }

#endif
        }
    }
}