using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class NetworkEmulatorRttTest : AbstractTest
    {
        private long changedValue;

        [Test]
        public void Test()
        {
            
        
            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.SetRttEmulation(500, 0);
            Thread.Sleep(50);
            CheetahLong.Increment(ref ObjectId, 2, 100);
        
            // пропускаем данные из комнаты
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            CheetahLong.SetListener(Listener);
            // команда не должна прийти,так как RTT = 200
            Debug.Log("changed value " + changedValue);
            Thread.Sleep(10);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(0, changedValue);
        
            // теперь команда должна прийти, так как прошло времени больше чем RTT
            Thread.Sleep(500);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(100, changedValue);
        
            // убираем задержки
            CheetahClient.SetCurrentClient(ClientA);
            CheetahClient.ResetEmulation();
            Thread.Sleep(20);
            CheetahLong.Increment(ref ObjectId, 2, 100);
        
            // команда должна прийти сразу же
            Thread.Sleep(100);
            CheetahClient.SetCurrentClient(ClientB);
            CheetahClient.Receive();
            Assert.AreEqual(200, changedValue);
        }

        

        [MonoPInvokeCallback(typeof(CheetahLong.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, long value)
        {
            changedValue = value;
        }
    }
}