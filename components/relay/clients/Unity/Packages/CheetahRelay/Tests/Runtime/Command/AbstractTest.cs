using System.Threading;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    public abstract class AbstractTest
    {
        protected ushort ClientA;
        protected ushort ClientB;
        protected CheetahObjectId ObjectId;

        [SetUp]
        public void SetUp()
        {
            var resultA = CheetahClient.CreateClient("127.0.0.1:5000", UserKeyGenerator.NextPublic(), ref UserKeyGenerator.Private, 0, out ClientA);
            var resultB = CheetahClient.CreateClient("127.0.0.1:5000", UserKeyGenerator.NextPublic(), ref UserKeyGenerator.Private, 0, out ClientB);

            Assert.True(resultA);
            Assert.True(resultB);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(ClientA);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusA));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusA);

            CheetahClient.SetCurrentClient(ClientB);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusB));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusB);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahObject.Create(55, 1, ref ObjectId);
            CheetahObject.Created(ref ObjectId);


            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.AttachToRoom();
            Thread.Sleep(100);
        }

        [TearDown]
        public void TearDown()
        {
            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.DestroyClient();

            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.DestroyClient();
        }
    }
}