using System.Threading;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class EventTest : AbstractTest
    {
        [Test]
        public void TestEventIncomeCommands()
        {
            // слушаем события определенного типа
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventFieldIdId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = clientB.Reader.GetEvents<DropMineEvent>(777, DropMineEventFieldIdId);
            var firstEvent = eventsStream[createdObject.ObjectId];
            Assert.AreEqual(dropMineEvent.MineId, firstEvent.MineId);
        }

        [Test]
        public void TestTargetEventIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventFieldIdId, memberB.UserId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = clientB.Reader.GetEvents<DropMineEvent>(777, DropMineEventFieldIdId);
            Assert.AreEqual(dropMineEvent.MineId, eventsStream[createdObject.ObjectId].MineId);
        }
    }
}