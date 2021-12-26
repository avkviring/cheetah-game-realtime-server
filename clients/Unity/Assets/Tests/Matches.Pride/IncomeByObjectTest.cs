using System.Collections;
using Cheetah.Matches.Relay.Income.ByObject;
using Cheetah.Matches.Relay.Income.ByTemplate;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Pride.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Pride
{
    public class IncomeByObjectTest: AbstractTest
    {

        [UnityTest]
        public IEnumerator TestEventIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события определенного типа
            var collector = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientB, createdObject.ObjectId,DropMineEventId);
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            createdObject.SendEvent(DropMineEventId, ref dropMineEvent);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream= collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA, actual.commandCreator);
        }

        [UnityTest]
        public IEnumerator TestTargetEventIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события определенного типа
            var collector = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientB, createdObject.ObjectId, DropMineEventId);
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            createdObject.SendEvent(DropMineEventId, memberB, ref dropMineEvent);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream= collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestStructureIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события определенного типа
            var collector = new StructureIncomeByObjectCommandCollector<TurretsParamsStructure>(clientB, createdObject.ObjectId,TurretsParamsFieldId);
            // изменяем структуру
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            createdObject.SetStructure(TurretsParamsFieldId, ref turretsParams);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var structuresStream= collector.GetStream();
            var actual = structuresStream.GetItem(0);
            var turretsParamsStructure = actual.value;
            Assert.AreEqual(turretsParams.Damage, turretsParamsStructure.Damage);
            Assert.AreEqual(turretsParams.Speed, turretsParamsStructure.Speed);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestLongIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события определенного типа
            var collector = new LongIncomeByObjectCommandCollector(clientB, createdObject.ObjectId, HealFieldId);
            // изменяем значение
            createdObject.SetLong(HealFieldId, 7799);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual( 7799, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        
        [UnityTest]
        public IEnumerator TestDoubleIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события определенного типа
            var collector = new DoubleIncomeByObjectCommandCollector(clientB, createdObject.ObjectId, HealFieldId);
            // изменяем значение
            createdObject.SetDouble(HealFieldId, 77.99);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual( 77.99, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
    }
}