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
            var helper = new AuthHelper();
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", helper.GetNextUserId(), helper.RoomId, ref helper.PrivateKey, 0,
                out var clientId));
            Assert.True(clientId > 0);
        }

        [Test]
        public void ShouldConnect()
        {
            var helper = new AuthHelper();
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", helper.GetNextUserId(), helper.RoomId, ref helper.PrivateKey, 0,
                out var clientId));
            Thread.Sleep(100);
            Assert.True(CheetahClient.GetConnectionStatus(out var status));
            Assert.AreEqual(CheetahConnectionStatus.Connected, status);
        }

        [Test]
        public void ShouldGetStatistics()
        {
            var helper = new AuthHelper();
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", helper.GetNextUserId(), helper.RoomId, ref helper.PrivateKey, 0,
                out var clientId));
            Thread.Sleep(100);
            CheetahClient.GetConnectionStatus(out var status);
            Assert.AreEqual(status, CheetahConnectionStatus.Connected);
            CheetahClient.GetStatistics(out var statistics);
            Assert.True(statistics.LastFrameId > 0);
        }


        [TearDown]
        public void TearDown()
        {
            CheetahClient.DestroyClient();
        }
    }
}