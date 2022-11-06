using System;
using System.Linq;
using System.Threading;
using Cheetah.Matches.Realtime.DOA.Income.ByTemplate;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
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

            var createdObjectStreamA = new CreatedObjectByTemplateIncomeCommands(clientA, 1);
            var createdObjectStreamB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);

            // создаем объект на первом клиенте
            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildRoomObject();
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientA.Update();
            clientB.Update();
            // проверяем результат - объект должен загрузится на всех клиентов, даже на текущего
            var objectsClientA = createdObjectStreamA.GetStream();
            var objectsClientB = createdObjectStreamB.GetStream();

            Assert.AreEqual(objectsClientA.Count, 1);
            Assert.AreEqual(objectsClientB.Count, 1);

            Assert.IsTrue(objectsClientA.First().cheetahObject.ObjectId.roomOwner);
            Assert.IsTrue(objectsClientB.First().cheetahObject.ObjectId.roomOwner);
        }

        [Test]
        public void ShouldCreateOneSingletonObject()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            Thread.Sleep(200);

            var createdObjectStream = new CreatedObjectByTemplateIncomeCommands(clientA, 1);

            // создаем объект на первом клиенте
            var someSingletonKey = new SomeSingletonKey { Key = new DateTime().Millisecond };
            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildSingletonRoomObject(ref someSingletonKey);
            clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).BuildSingletonRoomObject(ref someSingletonKey);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientA.Update();
            Thread.Sleep(200);

            // в итоге должен создаться только один объект
            var objectsClientA = createdObjectStream.GetStream();
            Assert.AreEqual(objectsClientA.Count, 1);
        }
    }
}