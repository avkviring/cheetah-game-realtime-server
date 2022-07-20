#nullable enable
using System;
using System.Collections.Generic;
using Cheetah.Statistics.Events.Internal;
using UnityEngine;
using UnityEngine.Assertions;

namespace Cheetah.Statistics.Events
{
    /// <summary>
    /// Сохранение Error, Exceptions логов Unity на сервер
    /// 
    /// </summary>
    public class UnityDebugLogSender
    {
        public const int AllowedSpamCount = 5;
        public static int SilenceSpamPeriodInMs = 1000 * 60;
        private readonly StatisticsSession session;
        private readonly Dictionary<StackTraceKey, SpamInfo> spamProtector = new();

        private struct StackTraceKey
        {
            public int lineNumber;
            public string methodName;
        }

        private class SpamInfo
        {
            public int count;
            public long nextSendTime;
        }


        public UnityDebugLogSender(StatisticsSession session)
        {
            Assert.IsNotNull(session, "session != null");
            this.session = session;
            Application.logMessageReceived += OnLog;
        }


        private void OnLog(string condition, string stacktrace, LogType type)
        {
            if (type is LogType.Error or LogType.Exception)
            {
                Dictionary<string, string> labels = new()
                {
                    ["level"] = type.ToString().ToLower(),
                    ["type"] = "log"
                };

                var extractor = new StackTraceExtractor(stacktrace);
                if (extractor.HasLineNumber())
                {
                    labels["line"] = extractor.GetLineNumber().ToString();
                }

                if (extractor.HasFileName())
                {
                    labels["file"] = extractor.GetFileName();
                }

                if (extractor.HasMethodName())
                {
                    labels["method"] = extractor.GetMethodName();
                }

                var spamInfo = GetSpamInfo(extractor);
                labels["count"] = (spamInfo?.count).ToString();
                if (spamInfo == null || spamInfo.count < AllowedSpamCount || spamInfo.nextSendTime < DateTimeOffset.UtcNow.ToUnixTimeMilliseconds())
                {
                    if (spamInfo != null)
                    {
                        spamInfo.nextSendTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() + SilenceSpamPeriodInMs;
                    }

                    var message = condition + "\n" + stacktrace;
                    session.Send(message, labels);
                }
            }
        }

        private SpamInfo? GetSpamInfo(StackTraceExtractor extractor)
        {
            var key = new StackTraceKey();
            if (extractor.HasMethodName() && extractor.HasLineNumber())
            {
                key.methodName = extractor.GetMethodName();
                key.lineNumber = extractor.GetLineNumber();
            }
            else if (extractor.HasMethodName())
            {
                key.methodName = extractor.GetFileName();
            }
            else
            {
                return null;
            }

            spamProtector.TryGetValue(key, out var spamInfo);
            if (spamInfo == null)
            {
                spamInfo = new SpamInfo();
                spamProtector[key] = spamInfo;
            }

            spamInfo.count++;

            return spamInfo;
        }
    }
}