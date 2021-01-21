using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    /// <summary>
    /// Тестируем правильность вызова FFI методов
    /// Сетевую часть не тестируем - так как для этого есть интеграционные тесты в rust клиенте
    /// </summary>
    public class CheetahClientTest
    {
        private string trace;

        [Test]
        public void Test()
        {
            CheetahClient.EnableTestMode(TestModeListener);
            var buffer = new CheetahBuffer().Add(3).Add(5);
            
            CheetahClient.CreateClient("127.0.0.1:5050", 5, 10, ref buffer, 15, out var clientId);
            Assert.AreEqual(trace, "create_client \"127.0.0.1:5050\" 5 10 15 [3, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]");

            CheetahClient.SetCurrentClient(clientId);
            Assert.AreEqual(trace, "set_current_client "+clientId);

            CheetahClient.SetChannelType(CheetahClient.ChannelType.ReliableUnordered, 100);
            Assert.AreEqual(trace, "set_channel ReliableUnordered 100");
            
            CheetahClient.SetDropEmulation(55, 11);
            Assert.AreEqual(trace, "set_drop_emulation 55.0 11");
            
            CheetahClient.SetRttEmulation(55, 11);
            Assert.AreEqual(trace, "set_rtt_emulation 55 11.0");
            
            CheetahClient.Receive();
            Assert.AreEqual(trace, "receive");
            
            CheetahClient.GetStatistics(out var cheetahStatistics);
            Assert.AreEqual(trace, "get_statistics Statistics { last_frame_id: 16, rtt_in_ms: 0, average_retransmit_frames: 0 }");
            
            CheetahClient.ResetEmulation();
            Assert.AreEqual(trace, "reset_emulation");
            
            CheetahClient.AttachToRoom();
            Assert.AreEqual(trace, "attach_to_room");
            
            CheetahClient.DetachFromRoom();
            Assert.AreEqual(trace, "send_command DetachFromRoom");
            
            CheetahClient.GetConnectionStatus(out var status);
            Assert.AreEqual(trace, "get_connection_status Connecting");
            
            var objectId = new CheetahObjectId();
            objectId.id = 100;
            CheetahClient.SetSourceObjectToMeta(ref objectId);
            Debug.Log(trace);
            Assert.AreEqual(trace, "set_source_object_to_meta GameObjectIdFFI { id: 100, room_owner: false, user_id: 0 }");
            
            CheetahClient.DestroyClient();
            Assert.AreEqual(trace, "destroy_client");
            
            
        }

        [MonoPInvokeCallback(typeof(CheetahClient.TestModeListener))]
        private void TestModeListener(string trace)
        {
            this.trace = trace;
        }
    }
}