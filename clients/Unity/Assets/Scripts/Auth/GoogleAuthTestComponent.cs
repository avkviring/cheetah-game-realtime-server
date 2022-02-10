using Cheetah.Accounts;
#if UNITY_ANDROID
using Cheetah.Accounts.Google;
#endif
using Cheetah.Platform;
using Grpc.Core;
using UnityEngine;
using UnityEngine.UI;

namespace Auth
{
    public class GoogleAuthTestComponent : MonoBehaviour
    {
        private const string AndroidWebClientId = "663521173650-gkgrl7aouifjag0j5do14pul1hdqvosm.apps.googleusercontent.com";

        [SerializeField] private Text resultText;
        [SerializeField] private Button androidLoginButton;
        private ClusterConnector clusterConnector = new ClusterConnector("test.dev.cheetah.games", 443, true);

#if UNITY_ANDROID
        private void Start()
        {
            androidLoginButton.onClick.AddListener(OnAndroidLogin);
        }


        private async void OnAndroidLogin()
        {
            try
            {
                // вначале используем сохраненый токен для авторизации
                // если такой токен есть - то нам не потребуется повторный вызов
                // внешней авторизации
                var jwt = new JWTUserAuthenticator();
                var user = await jwt.Login(clusterConnector);
                if (user != null)
                {
                    resultText.text = "Login with StoredPlayerAuthenticator";
                }
                else
                {
                    // сохраненного токена нет или он не валиден
                    // необходима внешняя авторизация
                    try
                    {
                        var androidAuthenticator = new GoogleAuthenticator(AndroidWebClientId);
                        var result = await androidAuthenticator.LoginOrRegister(clusterConnector);
                        user = result.User;
                        jwt.Store(user);
                        resultText.text = "Login with AndroidPlayerAuthenticator";
                    }
                    catch (GoogleAuthenticateException e)
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
        }
#endif
    }
}