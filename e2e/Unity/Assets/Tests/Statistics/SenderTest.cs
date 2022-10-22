using System;
using System.Collections;
using System.Collections.Generic;
using Cheetah.Platform;
using Cheetah.Platform.Tests;
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
            var connectorFactory = new KubernetesOrDockerConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;
            var session = new StatisticsSession(clusterConnector);
            var eventsSender = new EventsSender(session);
            eventsSender.Send("test");
            eventsSender.Send("play", new Dictionary<string, string>()
            {
                ["user"] = "Петя"
            });
        }

        [UnityTest]
        public IEnumerator TestLog()
        {
            var connectorFactory = new KubernetesOrDockerConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;
            var session = new StatisticsSession(clusterConnector);
            new UnityDebugLogSender(session);
            LogAssert.ignoreFailingMessages = true;
            Debug.LogError("it is error");
            Debug.LogException(new NullReferenceException());
        }

        [TearDown]
        public async void TearDown()
        {
            await clusterConnector.Destroy();
        }
    }
}