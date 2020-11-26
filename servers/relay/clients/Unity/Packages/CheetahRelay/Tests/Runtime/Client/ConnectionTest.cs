using System.Collections;
using System.Threading;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class ConnectionTest
    {
        [Test]
        public void ShouldCreateClient()
        {
            var user = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", user.publicKey, ref user.privateKey, out var clientId));
            Assert.True(clientId > 0);
        }

        [Test]
        public void ShouldConnect()
        {
            var user = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", user.publicKey, ref user.privateKey, out var clientId));
            Thread.Sleep(100);
            Assert.True(ClientCommands.GetConnectionStatus(out var status));
            Assert.AreEqual(ConnectionStatus.Connected, status);
        }


        [TearDown]
        public void TearDown()
        {
            ClientCommands.DestroyClient();
        }
    }
}