using System.Collections;
using Cheetah.Matches.Realtime.DOA.Income.ByTemplate;
using NUnit.Framework;
using Shared;
using Tests.Matches.Realtime.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime
{
    public class StressTest : AbstractTest
    {
        [UnityTest]
        public IEnumerator ShouldCreateLotOfObjects()
        {
            const int CountObjects = 1000;
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            clientB.Update();
            yield return new WaitForSeconds(1);

            var createdObjectStreamB = new CreatedObjectByTemplateIncomeCommands(clientB, 11);
            for (var i = 0; i < CountObjects; i++)
            {
                clientA.NewObjectBuilder(11, PlayerHelper.PlayerGroup).BuildRoomObject();
            }
            yield return new WaitForSeconds(1);
            clientA.Update();
            clientB.Update();
            Assert.AreEqual(createdObjectStreamB.GetStream().Count, CountObjects);
        }
    }
}