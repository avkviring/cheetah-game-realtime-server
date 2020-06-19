using System;
using Cheetach.Relay;
using UnityEngine;

namespace Runtime
{
    /**
     * Перенаправляет ошибки из нативной части клиента и сервера в консоль Unity    
     */
    public class LogGatewayComponent : MonoBehaviour
    {
#if UNITY_EDITOR
        private void Start()
        {
            LowLevelApi.InitLogger();
        }

        private static void ShowLog(LogLevel level, string log)
        {
            switch (level)
            {
                case LogLevel.Info:
                    Debug.Log(log);
                    break;
                case LogLevel.Warn:
                    Debug.LogWarning(log);
                    break;
                case LogLevel.Error:
                    Debug.LogError(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }

        void Update()
        {
            LowLevelApi.CollectLogs(ShowLog);
        }
#endif
    }
}