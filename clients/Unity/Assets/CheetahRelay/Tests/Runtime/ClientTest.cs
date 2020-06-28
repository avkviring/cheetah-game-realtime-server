using System.Collections;
using System.Globalization;
using CheetahRelay.Runtime.LowLevel;
using CheetahRelay.Runtime.LowLevel.External;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using Random = System.Random;

namespace CheetahRelay.Tests.Runtime
{
    public class ClientTest
    {
        private ServerCommandListenerStub _serverCommandListenerA;
        private RelayClientListenerStub _listenerA;
        private RelayClient _clientA;
        

        private ServerCommandListenerStub _serverCommandListenerB;
        private RelayClientListenerStub _listenerB;
        private RelayClient _clientB;

        [SetUp]
        public void SetUp()
        {
            var random = new Random();
            var roomHash = random.NextDouble().ToString(CultureInfo.InvariantCulture);

            _listenerA = new RelayClientListenerStub();
            _serverCommandListenerA = new ServerCommandListenerStub();
            _clientA = new RelayClient("127.0.0.1:5000",roomHash, "clientA", _listenerA, _serverCommandListenerA);

            _listenerB = new RelayClientListenerStub();
            _serverCommandListenerB = new ServerCommandListenerStub();
            _clientB = new RelayClient("127.0.0.1:5000",roomHash, "clientB", _listenerB, _serverCommandListenerB);
            
        }

        [UnityTest]
        public IEnumerator ShouldConnect()
        {
            var listener = new RelayClientListenerStub();
            var serverCommandListener = new ServerCommandListenerStub();
            var client = new RelayClient("127.0.0.1:5000","some_room", "clientA", listener, serverCommandListener);
            yield return new WaitForSeconds(1);
            client.Update();
            Assert.AreEqual(listener.commands.Count, 1);
            Assert.AreEqual(listener.commands[0], "OnConnected");
        }
        
        [UnityTest]
        public IEnumerator ShouldNotConnect()
        {
            var listener = new RelayClientListenerStub();
            var serverCommandListener = new ServerCommandListenerStub();
            var client = new RelayClient("127.0.0.1:9000","some_room", "clientA", listener, serverCommandListener);
            yield return new WaitForSeconds(1);
            client.Update();
            Assert.AreEqual(listener.commands.Count, 1);
            Assert.AreEqual(listener.commands[0], "OnDisconnect");
        }
        
        [UnityTest]
        public IEnumerator ShouldDisconnect()
        {
            var listener = new RelayClientListenerStub();
            var serverCommandListener = new ServerCommandListenerStub();
            var client = new RelayClient("127.0.0.1:5000","some_room", "clientA", listener, serverCommandListener);
            yield return new WaitForSeconds(1);
            client.Update();
            client.Close();
            Assert.AreEqual(listener.commands.Count, 2);
            Assert.AreEqual(listener.commands[0], "OnConnected");
            Assert.AreEqual(listener.commands[1], "OnDisconnect");
        }
        
        [UnityTest]
        public IEnumerator ShouldUploadGameObject()
        {
            var structDataA = new Bytes();
            structDataA.AddValue(0x64);
            structDataA.AddValue(0x65);
            
            var structDataB = new Bytes();
            structDataB.AddValue(0x71);
            structDataB.AddValue(0x73);

            _clientA
                .GetGameObjectBuilder()
                .SetAccessGroup(7)
                .AddFloatCounter(10, 10.5)
                .AddLongCounter(20, 30)
                .AddStruct(5, in structDataA)
                .AddStruct(105, in structDataB)
                .BuildAndSendToServer();
            
            yield return new WaitForSeconds(1);
            _clientB.Update();
            
            Assert.AreEqual(_serverCommandListenerB.Commands.Count, 1);
            Assert.AreEqual(_serverCommandListenerB.Commands[0], "OnObjectUploaded[0] f[10]=10.5  l[20]=30  s[105] = 2(7173) s[5] = 2(6465)");
        }

        [UnityTest]
        public IEnumerator ShouldSendEvent()
        {
            var objectId = _clientA
                .GetGameObjectBuilder()
                .SetAccessGroup(7)
                .BuildAndSendToServer();

            var eventData = new Bytes();
            eventData.AddValue(0x64);
            eventData.AddValue(0x65);

            _clientA.SendEventToServer(objectId, 10, eventData);

            yield return new WaitForSeconds(1);
            _clientB.Update();

            Assert.AreEqual(_serverCommandListenerB.Commands.Count, 2);
            Assert.AreEqual(_serverCommandListenerB.Commands[1], "OnEvent[0] event id = 10, data = Bytes[size = 2, data=(64 65)]");
        }

        [UnityTest]
        public IEnumerator ShouldUpdateStruct()
        {
            var objectId = _clientA
                .GetGameObjectBuilder()
                .SetAccessGroup(7)
                .BuildAndSendToServer();

            var structureData = new Bytes();
            structureData.AddValue(0x64);
            structureData.AddValue(0x65);

            _clientA.UpdateStructureOnServer(objectId, 10, structureData);

            yield return new WaitForSeconds(1);
            _clientB.Update();

            Assert.AreEqual(_serverCommandListenerB.Commands.Count, 2);
            Assert.AreEqual(_serverCommandListenerB.Commands[1], "OnStructureUpdated[0] structure id = 10, data = Bytes[size = 2, data=(64 65)]");
        }

        [UnityTest]
        public IEnumerator ShouldUpdateLongCounter()
        {
            var objectId = _clientA
                .GetGameObjectBuilder()
                .SetAccessGroup(7)
                .BuildAndSendToServer();

            _clientA.IncrementLongCounterOnServer(objectId, 10, 100);
            _clientA.IncrementLongCounterOnServer(objectId, 10, 200);
            _clientA.SetLongCounterOnServer(objectId, 10, 55);

            yield return new WaitForSeconds(1);
            _clientB.Update();

            Assert.AreEqual(_serverCommandListenerB.Commands.Count, 4);
            Assert.AreEqual(_serverCommandListenerB.Commands[1], "OnLongCounterUpdated[0] counterId = 10, value = 100");
            Assert.AreEqual(_serverCommandListenerB.Commands[2], "OnLongCounterUpdated[0] counterId = 10, value = 300");
            Assert.AreEqual(_serverCommandListenerB.Commands[3], "OnLongCounterUpdated[0] counterId = 10, value = 55");
        }

        [UnityTest]
        public IEnumerator ShouldUpdateFloatCounter()
        {
            var objectId = _clientA
                .GetGameObjectBuilder()
                .SetAccessGroup(7)
                .BuildAndSendToServer();

            _clientA.IncrementFloatCounterOnServer(objectId, 10, 100.1);
            _clientA.IncrementFloatCounterOnServer(objectId, 10, 200.2);
            _clientA.SetFloatCounterOnServer(objectId, 10, 55.55);

            yield return new WaitForSeconds(1);
            _clientB.Update();

            Assert.AreEqual(_serverCommandListenerB.Commands.Count, 4);
            Assert.AreEqual(_serverCommandListenerB.Commands[1], "OnFloatCounterUpdated[0] counterId = 10, value = 100.1");
            Assert.AreEqual(_serverCommandListenerB.Commands[2], "OnFloatCounterUpdated[0] counterId = 10, value = 300.3");
            Assert.AreEqual(_serverCommandListenerB.Commands[3], "OnFloatCounterUpdated[0] counterId = 10, value = 55.55");
        }


        [TearDown]
        public void TearDown()
        {
            _clientA.Close();
            _clientB.Close();
        }
    }
}