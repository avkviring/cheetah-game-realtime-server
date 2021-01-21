using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    /// <summary>
    /// Тестируем правильность вызова FFI методов
    /// Сетевую часть не тестируем - так как для этого есть интеграционные тесты в rust клиенте
    /// </summary>
    public class CheetahEventTest
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

            CheetahEvent.Send(ref objectId, 10, ref buffer);
            Assert.AreEqual(trace, "send_command Event(EventCommand { object_id: GameObjectId { owner: User(0), id: 100 }, field_id: 10, event: [3, 5] })");

            CheetahEvent.SetListener(Listener);
            Assert.AreEqual(trace, "set_event_listener");
            Assert.True(listenerInvoked);
        }

        [MonoPInvokeCallback(typeof(CheetahClient.TestModeListener))]
        private void TestModeListener(string trace)
        {
            this.trace = trace;
        }

        [MonoPInvokeCallback(typeof(CheetahEvent.Listener))]
        private void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer buffer)
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
        }
    }
}