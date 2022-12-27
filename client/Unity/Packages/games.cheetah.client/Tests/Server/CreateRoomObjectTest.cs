using System;
using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using Games.Cheetah.Client.Tests.Server.Types;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class CreateRoomObjectTest : AbstractTest
    {
        [Test]
        public void ShouldCreateRoomObjects()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            clientB.Update();
            Thread.Sleep(200);

            // создаем объект на первом клиенте

            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildRoomObject();
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientA.Update();
            clientB.Update();

            // проверяем результат - объект должен загрузится на всех клиентов, даже на текущего
            var objectsClientA = clientA.Reader.GetCreatedObjectsInCurrentUpdate(1);
            var objectsClientB = clientB.Reader.GetCreatedObjectsInCurrentUpdate(1);

            Assert.AreEqual(objectsClientA.Count, 1);
            Assert.AreEqual(objectsClientB.Count, 1);

            Assert.IsTrue(objectsClientA.First().NetworkObject.ObjectId.IsRoomOwner);
            Assert.IsTrue(objectsClientB.First().NetworkObject.ObjectId.IsRoomOwner);
        }

        [Test]
        public void ShouldCreateOneSingletonObject()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            Thread.Sleep(200);


            // создаем объект на первом клиенте
            var someSingletonKey = new SomeSingletonKey { Key = new DateTime().Millisecond };
            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildSingletonRoomObject(ref someSingletonKey);
            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildSingletonRoomObject(ref someSingletonKey);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientA.Update();

            // в итоге должен создаться только один объект
            var objectsClientA = clientA.Reader.GetCreatedObjectsInCurrentUpdate(1);
            Assert.AreEqual(objectsClientA.Count, 1);
        }
    }
}