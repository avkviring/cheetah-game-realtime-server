using UnityEngine;

namespace Games.Cheetah.Client.Logger
{
    /**
     * Перенаправляет ошибки из нативной части клиента и сервера в консоль Unity    
     */
    public class LogGatewayComponent : MonoBehaviour
    {
#if UNITY_EDITOR
        private void Start()
        {
            LoggerGateway.Init();
        }

        private void Update()
        {
            LoggerGateway.CollectLogs(true);
        }
#endif
    }
}