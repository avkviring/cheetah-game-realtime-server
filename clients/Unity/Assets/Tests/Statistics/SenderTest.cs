using System.Collections;
using System.Collections.Generic;
using Cheetah.Platform;
using Cheetah.Statistics.Events;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Statistics
{
    public class SenderTest
    {
        private ClusterConnector clusterConnector;

        [UnityTest]
        public IEnumerator TestEvent()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            var sender = new EventsSender(clusterConnector);
            sender.SendEvent("test");
            sender.SendEvent("play", new Dictionary<string, string>()
            {
                ["user"] = "Петя"
            });
        }

        [UnityTest]
        public IEnumerator TestLog()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            var sender = new LogsSender(clusterConnector);
            sender.SendLog(LogType.Error, "it is error", "some stack trace");
            Debug.LogWarning("it is warning");
        }

        [TearDown]
        public async void TearDown()
        {
            await clusterConnector.Destroy();
        }
    }
}