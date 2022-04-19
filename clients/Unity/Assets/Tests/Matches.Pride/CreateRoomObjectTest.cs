using System.Collections;
using Cheetah.Matches.Relay.Income.ByTemplate;
using NUnit.Framework;
using Shared;
using Tests.Matches.Pride.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Pride
{
    public class CreateRoomObjectTest : AbstractTest
    {
        [UnityTest]
        public IEnumerator Test()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.AttachToRoom();
            clientB.AttachToRoom();
            clientA.Update();
            clientB.Update();

            var createdObjectStreamA = new CreatedObjectByTemplateIncomeCommands(clientA, 1);
            var createdObjectStreamB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);

            // создаем объект на первом клиенте
            clientA.NewRoomObjectBuilder(1, UserHelper.UserGroup).Build();
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientA.Update();
            clientB.Update();
            // проверяем результат
            Assert.AreEqual(createdObjectStreamA.GetStream().Count, 1);
            Assert.AreEqual(createdObjectStreamB.GetStream().Count, 1);
        }
    }
}