using CheetahRelay.Runtime.LowLevel;
using UnityEngine;

namespace CheetahRelay.Runtime.Component
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
            LoggerGateway.CollectLogs();
        }
#endif
    }
}