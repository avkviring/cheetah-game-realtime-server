using Games.Cheetah.Auth.External.Google;
using GooglePlayGames;
using GooglePlayGames.BasicApi;
using Grpc.Core;
using UnityEngine;
using UnityEngine.Serialization;
using UnityEngine.UI;

namespace games.cheetah.auth
{
    public class AndroidAuth : MonoBehaviour
    {
        
        public Text googleToken;
        public Text cerberusToken;
        
        private static bool first = true;


        private void Start()
        {
            if (!first)
            {
                return;
            }

            first = false;

            Debug.Log("AndroidAuth Starting auth");
            PlayGamesClientConfiguration config =
                new PlayGamesClientConfiguration.Builder("663521173650-gkgrl7aouifjag0j5do14pul1hdqvosm.apps.googleusercontent.com")
                    .RequestIdToken()
                    .Build();
            PlayGamesPlatform.DebugLogEnabled = true;
            PlayGamesPlatform.InitializeInstance(config);
            PlayGamesPlatform.Activate();            
            
            Debug.Log("AndroidAuth Runtime platform "+Application.platform);
            
            PlayGamesPlatform.Instance.Authenticate(SignInInteractivity.CanPromptOnce, (result) =>
            {

                googleToken.text = PlayGamesPlatform.Instance.GetIdToken();
                Debug.Log("AndroidAuth Authenticate " + result);
                Debug.Log("AndroidAuth Token Id " + PlayGamesPlatform.Instance.GetIdToken());
                Debug.Log("AndroidAuth Token User Id " + PlayGamesPlatform.Instance.GetUserId());
                Debug.Log("AndroidAuth Token Display Name " + PlayGamesPlatform.Instance.GetUserDisplayName());
                
                var channel = new Channel("192.168.212.97:6200", ChannelCredentials.Insecure);
                var client = new Games.Cheetah.Auth.External.Google.Google.GoogleClient(channel);
                var registryOrLoginRequest = new RegistryOrLoginRequest();
                registryOrLoginRequest.DeviceId = "device-id";
                registryOrLoginRequest.GoogleToken = PlayGamesPlatform.Instance.GetIdToken();
                var tokens = client.RegistryOrLogin(registryOrLoginRequest);
                Debug.Log("Session token "+tokens.Session);
                cerberusToken.text = tokens.Session;


            });
        }
    }
}