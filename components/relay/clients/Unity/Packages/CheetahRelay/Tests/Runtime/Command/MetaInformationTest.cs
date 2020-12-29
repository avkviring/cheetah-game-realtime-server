using System.Threading;
using AOT;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class MetaInformationTest : AbstractTest
    {
        private CheetahObjectId sourceFromMeta;

        [Test]
        public void Test()
        {
            CheetahClient.SetCurrentClient(ClientB);
            CheetahLong.SetListener(Listener);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahLong.Set(ref ObjectId, 1, 500);
            var source = new CheetahObjectId {id = 55, userId = 100};
            CheetahClient.SetSourceObjectToMeta(ref source);
            CheetahLong.Increment(ref ObjectId, 1, 100);
            Thread.Sleep(100);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(sourceFromMeta, source);
            
            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.SetSourceObjectToMeta(ref CheetahObjectId.Empty);
            CheetahLong.Increment(ref ObjectId, 1, 100);
            Thread.Sleep(100);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(sourceFromMeta, CheetahObjectId.Empty);
            
        }

        [MonoPInvokeCallback(typeof(CheetahLong.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            sourceFromMeta = meta.sourceObject;
        }
    }
}