// using System.Threading;
// using Games.Cheetah.Client.DOA.Income.ByTemplate;
// using NUnit.Framework;
// using Shared;
// using Tests.Matches.Realtime.Helpers;
//
// namespace Tests.Matches.Realtime
// {
//     public class StressTest : AbstractTest
//     {
//         [Test]
//         public void ShouldCreateLotOfObjects()
//         {
//             const int CountObjects = 1000;
//             // загружаем объекты комнаты - они нам не интересны
//             clientA.Update();
//             clientB.Update();
//             Thread.Sleep(1000);
//
//             var createdObjectStreamB = new CreatedObjectByTemplateIncomeCommands(clientB, 11);
//             for (var i = 0; i < CountObjects; i++)
//             {
//                 clientA.NewObjectBuilder(11, PlayerHelper.PlayerGroup).BuildRoomObject();
//             }
//
//             Thread.Sleep(1000);
//             clientA.Update();
//             clientB.Update();
//             Assert.AreEqual(createdObjectStreamB.GetStream().Count, CountObjects);
//         }
//     }
// }