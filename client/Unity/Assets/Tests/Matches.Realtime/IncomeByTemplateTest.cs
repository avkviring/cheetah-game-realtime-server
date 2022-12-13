using System.Threading;
using Games.Cheetah.Client.DOA.Income.ByObject;
using Games.Cheetah.Client.DOA.Income.ByTemplate;
using Games.Cheetah.Client.Types;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class IncomeByTemplateTest : AbstractTest
    {
        [Test]
        public void TestCreatedObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new CreatedObjectByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var objectBuilder = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup);
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            objectBuilder.SetStructure(TurretsParamsFieldId, in turretsParams);
            var createdObject = objectBuilder.Build();
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var createdObjectStream = collector.GetStream();
            var cheetahObjectConstructor = createdObjectStream[0];
            Assert.AreEqual(createdObject.ObjectId, cheetahObjectConstructor.cheetahObject.ObjectId);

            // проверяем структуру
            var incomeTurretsParams = new TurretsParamsStructure();
            cheetahObjectConstructor.GetStruct(TurretsParamsFieldId, ref incomeTurretsParams);
            Assert.AreEqual(turretsParams, incomeTurretsParams);
        }


        [Test]
        public void TestDeleteObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new DeletedObjectByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            clientA.Writer.Delete(in createdObject.ObjectId);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var deletedObjectStream = collector.GetStream();
            Assert.AreEqual(createdObject, deletedObjectStream.GetItem(0));
        }

        /// <summary>
        /// Проверяем посылку события от удаленного клиента по объекту созданному локальным
        /// 
        /// </summary>
        /// <returns></returns>
        //[Test]
        public void TestEventBySelfObjectIncomeCommands()
        {
            // слушаем создание объектов
            var collectorB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события 
            var eventCollectorA = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientA, createdObject.ObjectId, DropMineEventId);
            // ждем отправки команды
            Thread.Sleep(1000);
            // прием команды
            clientB.Update();

            var incomeEvent = collectorB.GetStream()[0];
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientB.Writer.SendEvent(in incomeEvent.cheetahObject.ObjectId, DropMineEventId, in dropMineEvent);

            // ждем отправки команды
            Thread.Sleep(1000);
            // прием команды
            clientA.Update();
            // проверяем результат
            var eventsStream = eventCollectorA.GetStream();
            Assert.AreEqual(1, eventsStream.Count);
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberB.UserId, actual.commandCreator);
        }

        [Test]
        public void TestDeleteFieldIncomeCommands()
        {
            const ushort fieldId = 1000;
            // слушаем создание новых объектов на втором клиенте
            var collector = new DeletedFieldByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            clientB.Writer.SetLong(in createdObject.ObjectId, fieldId, 5);
            clientB.Writer.DeleteField(createdObject.ObjectId, FieldType.Long, fieldId);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var deletedField = stream.GetItem(0);
            Assert.AreEqual(FieldType.Long, deletedField.fieldType);
            Assert.AreEqual(fieldId, deletedField.fieldId);
            Assert.AreEqual(createdObject.ObjectId, deletedField.cheetahObject.ObjectId);
        }
    }
}