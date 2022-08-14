using System.Collections;
using Cheetah.Matches.Realtime.DOA.Income.ByObject;
using Cheetah.Matches.Realtime.DOA.Income.ByTemplate;
using Cheetah.Matches.Realtime.Types;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime
{
    public class IncomeByTemplateTest : AbstractTest
    {
        [UnityTest]
        public IEnumerator TestCreatedObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new CreatedObjectByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var objectBuilder = clientA.NewObjectBuilder(777, UserHelper.UserGroup);
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            objectBuilder.SetStructure(TurretsParamsFieldId, ref turretsParams);
            var createdObject = objectBuilder.Build();
            // ждем отправки команды
            yield return new WaitForSeconds(1);
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


        [UnityTest]
        public IEnumerator TestDeleteObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new DeletedObjectByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, UserHelper.UserGroup).Build();
            createdObject.Delete();
            // ждем отправки команды
            yield return new WaitForSeconds(1);
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
        [UnityTest]
        public IEnumerator TestEventBySelfObjectIncomeCommands()
        {
            // слушаем создание объектов
            var collectorB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);
            ;
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, UserHelper.UserGroup).Build();
            // слушаем события 
            var eventCollectorA = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientA, createdObject.ObjectId, DropMineEventId);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();

            var incomeEvent = collectorB.GetStream()[0];
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            incomeEvent.cheetahObject.SendEvent(DropMineEventId, ref dropMineEvent);

            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientA.Update();
            // проверяем результат
            var eventsStream = eventCollectorA.GetStream();
            Assert.AreEqual(1, eventsStream.Count);
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberB, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestDeleteFieldIncomeCommands()
        {
            const ushort fieldId = 1000;
            // слушаем создание новых объектов на втором клиенте
            var collector = new DeletedFieldByTemplateIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, UserHelper.UserGroup).Build();
            createdObject.SetLong(fieldId, 5);
            createdObject.DeleteField(fieldId, FieldType.Long);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
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