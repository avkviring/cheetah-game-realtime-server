using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class NetworkEmulatorTest : AbstractTest
    {
        private long changedValue;

        [Test]
        public void TestRttEmulation()
        {
            changedValue = 0;
            CheetahClient.SetCurrentClient(ClientB);
            CheetahLong.SetListener(Listener);

            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.SetRttEmulation(200, 0);
            Thread.Sleep(10);
            ushort fieldId = 2;
            CheetahLong.Increment(ref ObjectId, fieldId, 100);

            // команда не должна прийти,так как RTT = 200
            Debug.Log("changed value " + changedValue);
            Thread.Sleep(10);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(0, changedValue);

            // теперь команда должна прийти, так как прошло времени больше чем RTT
            Thread.Sleep(200);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(100, changedValue);

            // убираем задержки
            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.ResetEmulation();
            Thread.Sleep(20);
            CheetahLong.Increment(ref ObjectId, fieldId, 100);

            // команда должна прийти сразу же
            Thread.Sleep(50);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(200, changedValue);
        }

        [Test]
        public void TestDropEmulation()
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