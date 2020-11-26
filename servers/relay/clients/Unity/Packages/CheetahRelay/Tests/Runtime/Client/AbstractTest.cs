using System.Threading;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    public abstract class AbstractTest
    {
        protected ushort clientA;
        protected ushort clientB;
        protected CheetahObjectId objectId;

        [SetUp]
        public void SetUp()
        {
            CheetahTestUserGenerator.UserKeys userA = CheetahTestUserGenerator.Generate();
            CheetahTestUserGenerator.UserKeys userB = CheetahTestUserGenerator.Generate();
            var resultA = CheetahClient.CreateClient("127.0.0.1:5000", userA.publicKey, ref userA.privateKey, 0, out clientA);
            var resultB = CheetahClient.CreateClient("127.0.0.1:5000", userB.publicKey, ref userB.privateKey, 0, out clientB);

            Assert.True(resultA);
            Assert.True(resultB);
            Thread.Sleep(100);


            CheetahClient.SetCurrentClient(clientA);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusA));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusA);

            CheetahClient.SetCurrentClient(clientB);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusB));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusB);

            CheetahClient.SetCurrentClient(clientA);
            var builder = new CheetahObjectBuilder();
            builder.SetTemplate(55);
            builder.SetAccessGroup(1);
            objectId = (CheetahObjectId) builder.BuildAndSendToServer();

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.AttachToRoom();
            Thread.Sleep(100);
        }

        [TearDown]
        public void TearDown()
        {
            CheetahClient.SetCurrentClient(clientA);
            CheetahClient.DestroyClient();

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.DestroyClient();
        }
    }
}