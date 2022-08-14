using System;
using System.Collections;
using System.Linq;
using Cheetah.Matches.Realtime.DOA.Income.ByTemplate;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime
{
    public class CreateRoomObjectTest : AbstractTest
    {
        [UnityTest]
        public IEnumerator ShouldCreateRoomObjects()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            clientB.Update();
            yield return new WaitForSeconds(1);

            var createdObjectStreamA = new CreatedObjectByTemplateIncomeCommands(clientA, 1);
            var createdObjectStreamB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);

            // создаем объект на первом клиенте
            clientA.NewObjectBuilder(1, UserHelper.UserGroup).BuildRoomObject();
            // ждем отправки команды
            yield return new WaitForSeconds(1);
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

        [UnityTest]
        public IEnumerator ShouldCreateOneSingletonObject()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            yield return new WaitForSeconds(1);

            var createdObjectStream = new CreatedObjectByTemplateIncomeCommands(clientA, 1);

            // создаем объект на первом клиенте
            var someSingletonKey = new SomeSingletonKey { Key = new DateTime().Millisecond };
            clientA.NewObjectBuilder(1, UserHelper.UserGroup).BuildSingletonRoomObject(ref someSingletonKey);
            clientA.NewObjectBuilder(1, UserHelper.UserGroup).BuildSingletonRoomObject(ref someSingletonKey);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientA.Update();
            yield return new WaitForSeconds(1);

            // в итоге должен создаться только один объект
            var objectsClientA = createdObjectStream.GetStream();
            Assert.AreEqual(objectsClientA.Count, 1);
        }
    }
}