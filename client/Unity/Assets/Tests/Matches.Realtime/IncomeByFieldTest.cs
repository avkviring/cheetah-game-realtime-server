using System.Threading;
using Games.Cheetah.Client.DOA.Income.ByField;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class IncomeByFieldTest : AbstractTest
    {
        [Test]
        public void TestEventIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new EventIncomeByFieldCommandCollector<DropMineEvent>(clientB, DropMineEventId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestTargetEventIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new EventIncomeByFieldCommandCollector<DropMineEvent>(clientB, DropMineEventId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventId, memberB.UserId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestStructureIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new StructureIncomeByFieldCommandCollector<TurretsParamsStructure>(clientB, TurretsParamsFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем структуру
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            clientA.Writer.SetStructure(in createdObject.ObjectId, TurretsParamsFieldId, in turretsParams);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var structuresStream = collector.GetStream();
            var actual = structuresStream.GetItem(0);
            var turretsParamsStructure = actual.value;
            Assert.AreEqual(turretsParams.Damage, turretsParamsStructure.Damage);
            Assert.AreEqual(turretsParams.Speed, turretsParamsStructure.Speed);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestCompareAndSetStructureIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new StructureIncomeByFieldCommandCollector<TurretsParamsStructure>(clientB, TurretsParamsFieldId);

            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            var turretParamsA = new TurretsParamsStructure()
            {
                Damage = 2,
                Speed = 130
            };

            var turretParamsB = turretParamsA;
            turretParamsB.Speed = 100;

            var turretParamsC = turretParamsB;
            turretParamsC.Damage = 5;


            clientA.Writer.SetStructure(in createdObject.ObjectId, TurretsParamsFieldId, in turretParamsA);
            clientA.Writer.CompareAndSet(in createdObject.ObjectId, TurretsParamsFieldId, in turretParamsA, in turretParamsB);
            clientA.Writer.CompareAndSet(in createdObject.ObjectId, TurretsParamsFieldId, in turretParamsB, in turretParamsC, in turretParamsA);

            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();

            // проверяем результат
            var stream = collector.GetStream();
            var first = stream.GetItem(1);
            var second = stream.GetItem(2);
            Assert.AreEqual(100, first.value.Speed);
            Assert.AreEqual(5, second.value.Damage);
            Assert.AreEqual(memberA.UserId, first.commandCreator);
        }

        [Test]
        public void TestLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, ScoreFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.SetLong(in createdObject.ObjectId, ScoreFieldId, 7799L);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(7799, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestIncrementLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, ScoreFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.Increment(in createdObject.ObjectId, ScoreFieldId, 1001L);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(1001, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestCompareAndSetLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, ScoreFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.CompareAndSet(in createdObject.ObjectId, ScoreFieldId, 0L, 555L);
            clientA.Writer.CompareAndSet(in createdObject.ObjectId, ScoreFieldId, 555L, 1000L, true, 0L);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var first = stream.GetItem(0);
            var second = stream.GetItem(1);
            Assert.AreEqual(555, first.value);
            Assert.AreEqual(1000, second.value);
            Assert.AreEqual(memberA.UserId, first.commandCreator);
        }

        [Test]
        public void TestDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.SetDouble(in createdObject.ObjectId, HealFieldId, 77.99);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(77.99, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestIncrementDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.Increment(in createdObject.ObjectId, HealFieldId, 77.99);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(77.99, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }
    }
}