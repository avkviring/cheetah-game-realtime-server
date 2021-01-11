using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    /// <summary>
    /// Тестируем правильность вызова FFI методов
    /// Сетевую часть не тестируем - так как для этого есть интеграционные тесты в rust клиенте
    /// </summary>
    public class CheetahDoubleTest
    {
        private string trace;
        private bool listenerInvoked;

        [Test]
        public void Test()
        {
            CheetahClient.EnableTestMode(TestModeListener);
            var buffer = new CheetahBuffer().Add(3).Add(5);
            CheetahClient.CreateClient("127.0.0.1:5050", 5, 10, ref buffer, 15, out var clientId);
            var objectId = new CheetahObjectId();
            objectId.id = 100;
            
            CheetahDouble.Increment(ref objectId, 10, 100);
            Assert.AreEqual(trace, "send_command IncrementFloatCounter(IncrementFloat64C2SCommand { object_id: GameObjectId { owner: User(0), id: 100 }, field_id: 10, increment: 100.0 })");
            
            CheetahDouble.Set(ref objectId, 15, 555);
            Assert.AreEqual(trace, "send_command SetFloat(SetFloat64Command { object_id: GameObjectId { owner: User(0), id: 100 }, field_id: 15, value: 555.0 })");
            
            CheetahDouble.SetListener(Listener);
            Assert.AreEqual(trace, "set_float_value_listener");
            Assert.True(listenerInvoked);

        }

        [MonoPInvokeCallback(typeof(CheetahClient.TestModeListener))]
        private void TestModeListener(string trace)
        {
            this.trace = trace;
        }
        
        [MonoPInvokeCallback(typeof(CheetahDouble.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, double value)
        {
            this.listenerInvoked = true;
            Assert.AreEqual(meta.sourceUser, 15);
            Assert.AreEqual(meta.timestamp, 25);
            Assert.AreEqual(meta.sourceObject.id, 3);
            Assert.AreEqual(meta.sourceObject.roomOwner, false);
            Assert.AreEqual(meta.sourceObject.userId, 5);
            Assert.AreEqual(objectId.id, 5);
            Assert.AreEqual(objectId.roomOwner, false);
            Assert.AreEqual(objectId.userId, 77);
            Assert.AreEqual(fieldId, 77);
            Assert.AreEqual(value, 5);
        }
    }
}