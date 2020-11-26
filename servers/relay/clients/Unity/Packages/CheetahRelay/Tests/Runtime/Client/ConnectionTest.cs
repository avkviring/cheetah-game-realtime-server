using System.Threading;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class ConnectionTest
    {
        [Test]
        public void ShouldCreateClient()
        {
            var user = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", user.publicKey, ref user.privateKey, 0, out var clientId));
            Assert.True(clientId > 0);
        }

        [Test]
        public void ShouldConnect()
        {
            var user = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", user.publicKey, ref user.privateKey, 0, out var clientId));
            Thread.Sleep(100);
            Assert.True(ClientCommands.GetConnectionStatus(out var status));
            Assert.AreEqual(ConnectionStatus.Connected, status);
        }

        [Test]
        public void ShouldGetFrameId()
        {
            var user = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", user.publicKey, ref user.privateKey, 0, out var clientId));
            Thread.Sleep(100);
            Assert.True(ClientCommands.GetConnectionStatus(out var status));
            ClientCommands.GetFrameId(out var frameId);
            Assert.True(frameId > 0);
        }


        [TearDown]
        public void TearDown()
        {
            ClientCommands.DestroyClient();
        }
    }
}