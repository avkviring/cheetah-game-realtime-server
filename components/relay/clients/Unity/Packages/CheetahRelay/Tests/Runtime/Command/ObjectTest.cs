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
        private CheetahObjectId createObjectId;
        private CheetahObjectId createdObjectId;
        private ushort createdObjectTemplate;
        private CheetahObjectId deletedObjectId;

        [SetUp]
        public void SetUp()
        {
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", UserKeyGenerator.GetNextUserId(), 1,ref UserKeyGenerator.PrivateKey, 0, out clientA));
            Assert.True(CheetahClient.CreateClient("127.0.0.1:5000", UserKeyGenerator.GetNextUserId(), 1,ref UserKeyGenerator.PrivateKey,0,  out clientB));
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
            
            CheetahObjectId objectId = new CheetahObjectId();
            CheetahObject.Create(55, 1, ref objectId);
            CheetahObject.Created(ref objectId);
            
            Thread.Sleep(100);

            CheetahClient.SetCurrentClient(clientB);
            CheetahObject.SetCreateListener(OnCreate);
            CheetahObject.SetCreatedListener(OnCreated);
            CheetahClient.AttachToRoom();
            Thread.Sleep(100);
            CheetahClient.Receive();
            Assert.AreEqual(objectId, createObjectId);
            Assert.AreEqual(objectId, createdObjectId);
            Assert.AreEqual(55, createdObjectTemplate);
        }

        [MonoPInvokeCallback(typeof(CheetahObject.CreateListener))]
        private void OnCreate(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort template)
        {
            createObjectId = objectId;
            createdObjectTemplate = template;
        }
        
        [MonoPInvokeCallback(typeof(CheetahObject.CreatedListener))]
        private void OnCreated(ref CheetahCommandMeta meta, ref CheetahObjectId objectId)
        {
            createdObjectId = objectId; 
        }


        [Test]
        public void ShouldDeleteObject()
        {
            CheetahClient.SetCurrentClient(clientA);
            LoggerExternals.SetMaxLogLevel(CheetahLogLevel.Info);
            CheetahObjectId objectId = new CheetahObjectId();
            CheetahObject.Create(55, 1, ref objectId);
            CheetahObject.Created(ref objectId);
            Thread.Sleep(100);
            
            CheetahClient.SetCurrentClient(clientB);
            CheetahClient.AttachToRoom();
            CheetahObject.SetDeleteListener(OnDelete);
            Thread.Sleep(100);
            
            CheetahClient.SetCurrentClient(clientA);
            CheetahObject.Delete(ref objectId);
            Thread.Sleep(500);
            
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