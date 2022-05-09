using System.Collections;
using Cheetah.Platform;
using Cheetah.System.Compatibility;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine.TestTools;

namespace Tests.System
{
    public class CompatibilityTest
    {
        private ClusterConnector clusterConnector;

        [UnityTest]
        public IEnumerator Test()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;
            var checker = new CompatibilityChecker(clusterConnector);
            yield return Enumerators.Await(checker.Check("0.0.1"));
        }

        [TearDown]
        public async void TearDown()
        {
            await clusterConnector.Destroy();
        }
    }
}