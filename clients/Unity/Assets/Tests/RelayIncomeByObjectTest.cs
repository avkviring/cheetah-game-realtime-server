using System.Collections;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Income.ByObject;
using Cheetah.Matches.Relay.Income.ByTemplate;
using Cheetah.Platform;
using NUnit.Framework;
using Shared;
using Tests.Helpers;
using Tests.Types;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class RelayIncomeByObjectTest
    {
        private ClusterConnector clusterConnector;
        private CheetahClient clientA;
        
        private CheetahClient clientB;
        private uint memberA;
        private uint memberB;
        private const ushort TurretsParamsFieldId = 333;
        private const ushort DropMineEventId = 555;
        private const ushort HealFieldId = 1; 

         

        [UnitySetUp]
        public IEnumerator SetUp()
        {

            var codecRegistry = new CodecRegistry();
            codecRegistry.RegisterEventCodec(DropMineEventId, new DropMineEventCodec());
            codecRegistry.RegisterStructureCodec(TurretsParamsFieldId, new TurretsParamsStructureCodec());
            
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;
            
            // подключаем первого клиента
            var ticketA = PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector,"user_a");
            yield return Enumerators.Await(ticketA);
            memberA = ticketA.Result.UserId;
            clientA = ConnectToRelay(ticketA.Result,codecRegistry);
            clientA.AttachToRoom();

            // подключаем второрого клиента
            var ticketB = PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector,"user_b");
            yield return Enumerators.Await(ticketB);
            memberB = ticketB.Result.UserId;
            clientB = ConnectToRelay(ticketB.Result,codecRegistry);
            clientB.AttachToRoom();
            
            // полуаем сетевые команды, которые не надо учитывать в тестах
            yield return new WaitForSeconds(1);
            clientA.Update();
            clientB.Update();
            
        }
        
        
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
        
        /// <summary>
        /// Проверяем посылку события от удаленного клиента по объекту созданному локальным
        /// 
        /// </summary>
        /// <returns></returns>
        [UnityTest]
        public IEnumerator TestEventBySelfObjectIncomeCommands()
        {
            // слушаем создание объектов
            var collectorB = new CreatedObjectByTemplateIncomeCommands(clientB, 1);;
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            // слушаем события 
            var eventCollectorA = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientA, createdObject.ObjectId,DropMineEventId);
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
            var eventsStream= eventCollectorA.GetStream();
            Assert.AreEqual(1,eventsStream.Count);
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberB, actual.commandCreator);
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
       
        


        private static CheetahClient ConnectToRelay(TicketResponse ticket, CodecRegistry codecRegistry)
        {
            return new CheetahClient(ticket.RelayGameHost, ticket.RelayGamePort, ticket.UserId, ticket.RoomId, ticket.PrivateKey.ToByteArray(), codecRegistry);
        }

        [TearDown]
        public async void TearDown()
        {
            clientA.Destroy();
            clientB.Destroy();
            await clusterConnector.Destroy();
        }
    }
}