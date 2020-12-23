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
            CheetahClient.SetCurrentClient(ClientB);
            CheetahLong.SetListener(Listener);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahLong.Set(ref ObjectId, 1, 500);
            CheetahLong.Increment(ref ObjectId, 1, 100);
            CheetahLong.Increment(ref ObjectId, 1, 200);
            CheetahLong.CompareAndSet(ref ObjectId, 1, 800, 900, 0);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(changedValue, 900);
            Assert.AreEqual(changedField, 1);
            Assert.AreEqual(changedObjectId, ObjectId);
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