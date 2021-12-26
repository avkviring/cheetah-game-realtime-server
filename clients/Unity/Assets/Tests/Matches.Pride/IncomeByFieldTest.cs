using System.Collections;
using Cheetah.Matches.Relay.Income.ByField;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Pride.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Pride
{
    public class IncomeByFieldTest: AbstractTest
    {
        [UnityTest]
        public IEnumerator TestEventIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new EventIncomeByFieldCommandCollector<DropMineEvent>(clientB, DropMineEventId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
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
            // слушаем события определенного типа
            var collector = new EventIncomeByFieldCommandCollector<DropMineEvent>(clientB, DropMineEventId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
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
            // слушаем события определенного типа
            var collector = new StructureIncomeByFieldCommandCollector<TurretsParamsStructure>(clientB, TurretsParamsFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
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
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
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
        public IEnumerator TestIncrementLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.IncrementLong(HealFieldId, 1001);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(1001, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestCompareAndSetLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.CompareAndSet(HealFieldId, 0,555,0);
            // ждем отправки команды
            yield return new WaitForSeconds(1);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual( 555, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
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
        
        [UnityTest]
        public IEnumerator TestIncrementDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeByFieldCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.IncrementDouble(HealFieldId, 77.99);
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