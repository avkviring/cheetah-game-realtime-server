using System.Threading;
using AOT;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class LongValueTest : AbstractTest
    {
        private long changedValue;
        private CheetahObjectId changedObjectId;
        private ushort changedField;

        [Test]
        public void Test()
        {
            CheetahClient.SetCurrentClient(clientB);
            CheetahLong.SetListener(Listener);

            CheetahClient.SetCurrentClient(clientA);
            CheetahLong.Set(ref objectId, 1, 500);
            CheetahLong.Increment(ref objectId, 1, 100);
            CheetahLong.Increment(ref objectId, 1, 200);
            CheetahLong.CompareAndSet(ref objectId, 1, 800, 900, 0);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.Receive();
            Assert.AreEqual(changedValue, 900);
            Assert.AreEqual(changedField, 1);
            Assert.AreEqual(changedObjectId, objectId);
        }

        [MonoPInvokeCallback(typeof(CheetahLong.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            changedValue = value;
            changedObjectId = objectId;
            changedField = fieldId;
        }
    }
}