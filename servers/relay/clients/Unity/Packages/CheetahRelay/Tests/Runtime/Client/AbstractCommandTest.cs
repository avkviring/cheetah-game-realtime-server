using System.Collections;
using System.Threading;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace CheetahRelay.Tests
{
    public abstract class AbstractCommandTest
    {
        protected ushort clientA;
        protected ushort clientB;
        protected RelayObjectId objectId;

        [SetUp]
        public void SetUp()
        {
            
            UserKeys userA = TestUserGenerator.Generate();
            UserKeys userB = TestUserGenerator.Generate();
            var resultA = ClientCommands.CreateClient("127.0.0.1:5000", userA.publicKey, ref userA.privateKey, out clientA);
            var resultB = ClientCommands.CreateClient("127.0.0.1:5000", userB.publicKey, ref userB.privateKey, out clientB);
            
            Assert.True(resultA);
            Assert.True(resultB);
            Thread.Sleep(100);
            

            ClientCommands.SetCurrentClient(clientA);
            Assert.True(ClientCommands.GetConnectionStatus(out var statusA));
            Assert.AreEqual(ConnectionStatus.Connected, statusA);

            ClientCommands.SetCurrentClient(clientB);
            Assert.True(ClientCommands.GetConnectionStatus(out var statusB));
            Assert.AreEqual(ConnectionStatus.Connected, statusB);

            ClientCommands.SetCurrentClient(clientA);
            var builder = new RelayGameObjectBuilder();
            builder.SetTemplate(55);
            builder.SetAccessGroup(1);
            objectId = (RelayObjectId) builder.BuildAndSendToServer();

            ClientCommands.SetCurrentClient(clientB);
            AttachToRoomCommand.AttachToRoom();
            Thread.Sleep(100);
        }
        
        [TearDown]
        public void TearDown()
        {
            ClientCommands.SetCurrentClient(clientA);
            ClientCommands.DestroyClient();

            ClientCommands.SetCurrentClient(clientB);
            ClientCommands.DestroyClient();
        }
    }
}