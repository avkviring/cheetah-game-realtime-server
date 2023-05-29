using System;
using AOT;
using UnityEngine;

namespace Games.Cheetah.Client.Logger
{
    /**
     * Отбражение логов с клиента в Unity консоле
     */
    public class NetworkClientLogs
    {
        public static void Init()
        {
            LoggerExternals.InitLogger();
            LoggerExternals.SetMaxLogLevel(CheetahLogLevel.Info);
        }

        [MonoPInvokeCallback(typeof(LoggerExternals.LogCollector))]
        private static void ShowLog(CheetahLogLevel level, string log)
        {
            switch (level)
            {
                case CheetahLogLevel.Info:
                    Debug.Log("client:\t" + log);
                    break;
                case CheetahLogLevel.Warn:
                    Debug.LogWarning("client:\t" + log);
                    break;
                case CheetahLogLevel.Error:
                    Debug.LogError("client:\t" + log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }

        [MonoPInvokeCallback(typeof(LoggerExternals.LogCollector))]
        private static void SkipLog(CheetahLogLevel level, string log)
        {
        }

        public static void CollectLogs(bool showLog)
        {
            if (showLog)
            {
                LoggerExternals.CollectLogs(ShowLog);
            }
            else
            {
                LoggerExternals.CollectLogs(SkipLog);
            }
        }
    }
}