using System.Threading;
using AOT;
using NUnit.Framework;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class LongValueTest : AbstractCommandTest
    {
        private long changedValue;
        private RelayObjectId changedObjectId;
        private ushort changedField;

        [Test]
        public void Test()
        {
            ClientCommands.SetCurrentClient(clientB);
            LongValueCommands.SetListener(Listener);

            ClientCommands.SetCurrentClient(clientA);
            LongValueCommands.Set(ref objectId, 1, 500);
            LongValueCommands.Increment(ref objectId, 1, 100);
            LongValueCommands.Increment(ref objectId, 1, 200);
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientB);
            ClientCommands.Receive();
            Assert.AreEqual(changedValue, 800);
            Assert.AreEqual(changedField, 1);
            Assert.AreEqual(changedObjectId, objectId);
        }

        [MonoPInvokeCallback(typeof(LongValueCommands.Listener))]
        private void Listener(ref CommandMeta meta, ref RelayObjectId objectId, ushort fieldId, long value)
        {
            changedValue = value;
            changedObjectId = objectId;
            changedField = fieldId;
        }
    }
}