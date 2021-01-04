using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class NetworkEmulatorDropTest : AbstractTest
    {
        private long changedValue;
        
        [Test]
        public void Test()
        {
            CheetahClient.SetCurrentClient(ClientB);
            CheetahLong.SetListener(Listener);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.SetDropEmulation(0.3, 1);
            Thread.Sleep(50);
            var count = 100;
            var increment = 100;
            for (var i = 0; i < count; i++)
            {
                CheetahLong.Increment(ref ObjectId, 1, increment);
                Thread.Sleep(1);
            }

            Thread.Sleep(500);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.True(changedValue < count * increment);
            Assert.True(changedValue > 0);
        }


        [MonoPInvokeCallback(typeof(CheetahLong.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            changedValue = value;
        }
    }
}