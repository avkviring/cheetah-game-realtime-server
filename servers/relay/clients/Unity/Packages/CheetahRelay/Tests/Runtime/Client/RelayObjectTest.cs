using System;
using System.Collections;
using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class RelayObjectTest
    {
        private ushort clientA;
        private ushort clientB;
        private RelayObjectId createdObjectId;
        private ushort createdObectTemplate;
        private GameObjectFields createdObjectFields;
        private RelayObjectId deletedObjectId;

        [SetUp]
        public void SetUp()
        {
            UserKeys userA = TestUserGenerator.Generate();
            UserKeys userB = TestUserGenerator.Generate();
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", userA.publicKey, ref userA.privateKey, 0, out clientA));
            Assert.True(ClientCommands.CreateClient("127.0.0.1:5000", userB.publicKey, ref userB.privateKey,0,  out clientB));
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientA);
            Assert.True(ClientCommands.GetConnectionStatus(out var statusA));
            Assert.AreEqual(ConnectionStatus.Connected, statusA);

            ClientCommands.SetCurrentClient(clientB);
            Assert.True(ClientCommands.GetConnectionStatus(out var statusB));
            Assert.AreEqual(ConnectionStatus.Connected, statusB);
        }


        [Test]
        public void ShouldCreateObject()
        {
            ClientCommands.SetCurrentClient(clientA);
            var builder = new RelayGameObjectBuilder();
            builder.SetTemplate(55);
            builder.SetAccessGroup(1);
            builder.SetDouble(1, 100.0);
            builder.SetLong(2, 200);
            builder.SetStructure(3, new RelayBuffer().Add(123));
            var objectId = builder.BuildAndSendToServer();
            Assert.NotNull(objectId);
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientB);
            CreateObjectCommand.SetListener(OnCreate);
            AttachToRoomCommand.AttachToRoom();
            Thread.Sleep(100);
            ClientCommands.Receive();
            Assert.AreEqual(objectId, createdObjectId);
            Assert.AreEqual(builder.template, createdObectTemplate);
            Assert.AreEqual(builder.fields, createdObjectFields);
        }

        [MonoPInvokeCallback(typeof(CreateObjectCommand.Listener))]
        private void OnCreate(ref CommandMeta meta, ref RelayObjectId objectId, ushort template, ref GameObjectFields fields)
        {
            createdObjectId = objectId;
            createdObectTemplate = template;
            createdObjectFields = fields;
        }


        [Test]
        public void ShouldDeleteObject()
        {
            ClientCommands.SetCurrentClient(clientA);
            var builder = new RelayGameObjectBuilder();
            builder.SetAccessGroup(1);
            var objectId = (RelayObjectId) builder.BuildAndSendToServer();
            Debug.Log("objectId "+objectId);
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientB);
            AttachToRoomCommand.AttachToRoom();
            DeleteObjectCommand.SetListener(OnDelete);
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientA);
            DeleteObjectCommand.Delete(ref objectId);
            Thread.Sleep(100);

            ClientCommands.SetCurrentClient(clientB);
            ClientCommands.Receive();

            Assert.AreEqual(objectId, deletedObjectId);
        }

        [MonoPInvokeCallback(typeof(DeleteObjectCommand.Listener))]
        private void OnDelete(ref CommandMeta meta, ref RelayObjectId objectId)
        {
            deletedObjectId = objectId;
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