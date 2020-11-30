using System.Threading;
using AOT;
using NUnit.Framework;
using UnityEngine;

namespace CheetahRelay.Tests
{
    [TestFixture]
    public class ObjectTest
    {
        private ushort clientA;
        private ushort clientB;
        private CheetahObjectId createdObjectId;
        private ushort createdObectTemplate;
        private GameObjectFields createdObjectFields;
        private CheetahObjectId deletedObjectId;

        [SetUp]
        public void SetUp()
        {
            CheetahTestUserGenerator.UserKeys userA = CheetahTestUserGenerator.Generate();
            CheetahTestUserGenerator.UserKeys userB = CheetahTestUserGenerator.Generate();
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", userA.publicKey, ref userA.privateKey, 0, out clientA));
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", userB.publicKey, ref userB.privateKey,0,  out clientB));
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientA);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusA));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusA);

            CheetahClient.SetCurrentClient(clientB);
            Assert.True(CheetahClient.GetConnectionStatus(out var statusB));
            Assert.AreEqual(CheetahConnectionStatus.Connected, statusB);
        }


        [Test]
        public void ShouldCreateObject()
        {
            CheetahClient.SetCurrentClient(clientA);
            var builder = new CheetahObjectBuilder();
            builder.SetTemplate(55);
            builder.SetAccessGroup(1);
            builder.SetDouble(1, 100.0);
            builder.SetLong(2, 200);
            var cheetahBuffer = new CheetahBuffer().Add(123);
            builder.SetStructure(3, ref cheetahBuffer);
            var objectId = builder.BuildAndSendToServer();
            Assert.NotNull(objectId);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientB);
            CheetahObject.SetListener(OnCreate);
            CheetahClient.AttachToRoom();
            Thread.Sleep(100);
            CheetahClient.Receive();
            Assert.AreEqual(objectId, createdObjectId);
            Assert.AreEqual(builder.template, createdObectTemplate);
            Assert.AreEqual(builder.fields, createdObjectFields);
        }

        [MonoPInvokeCallback(typeof(CheetahObject.CreateListener))]
        private void OnCreate(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort template, ref GameObjectFields fields)
        {
            createdObjectId = objectId;
            createdObectTemplate = template;
            createdObjectFields = fields;
        }


        [Test]
        public void ShouldDeleteObject()
        {
            CheetahClient.SetCurrentClient(clientA);
            var builder = new CheetahObjectBuilder();
            builder.SetAccessGroup(1);
            var objectId = (CheetahObjectId) builder.BuildAndSendToServer();
            Debug.Log("objectId "+objectId);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.AttachToRoom();
            CheetahObject.SetListener(OnDelete);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientA);
            CheetahObject.Delete(ref objectId);
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.Receive();

            Assert.AreEqual(objectId, deletedObjectId);
        }

        [MonoPInvokeCallback(typeof(CheetahObject.DeleteListener))]
        private void OnDelete(ref CheetahCommandMeta meta, ref CheetahObjectId objectId)
        {
            deletedObjectId = objectId;
        }


        [TearDown]
        public void TearDown()
        {
            CheetahClient.SetCurrentClient(clientA);
            CheetahClient.DestroyClient();

            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.DestroyClient();
        }
    }
}