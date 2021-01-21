using System;
using AOT;
using UnityEngine;

namespace CheetahRelay
{
    /**
     * Отбражение логов с клиента в Unity консоле
     */
    public class LoggerGateway
    {
        public static void Init()
        {
            LoggerExternals.InitLogger();
        }

        [MonoPInvokeCallback(typeof(LoggerExternals.LogCollector))]
        private static void ShowLog(CheetahLogLevel level, string log)
        {
            switch (level)
            {
                case CheetahLogLevel.Info:
                    Debug.Log(log);
                    break;
                case CheetahLogLevel.Warn:
                    Debug.LogWarning(log);
                    break;
                case CheetahLogLevel.Error:
                    Debug.LogError(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }

        public static void CollectLogs()
        {
            LoggerExternals.CollectLogs(ShowLog);
        }
    }
}