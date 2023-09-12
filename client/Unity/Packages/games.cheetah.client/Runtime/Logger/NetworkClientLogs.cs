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
            LoggerExternals.SetMaxLogLevel(CheetahLogLevel.Error);
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
                case CheetahLogLevel.Trace:
                    Debug.Log("client:\t" + log);
                    break;
                case CheetahLogLevel.Debug:
                    Debug.Log("client:\t" + log);
                    break;
            }
        }
        
        public static void CollectLogs(bool showLog)
        {
            if (showLog)
            {
                LoggerExternals.CollectLogs(ShowLog);
            }
        }
    }
}