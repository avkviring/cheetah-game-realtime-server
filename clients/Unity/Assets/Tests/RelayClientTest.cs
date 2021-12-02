using System.Collections;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Income;
using Cheetah.Platform;
using NUnit.Framework;
using Shared;
using Tests.Helpers;
using Tests.Types;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class RelayClientTest
    {
        private ClusterConnector clusterConnector;
        private CheetahClient clientA;
        
        private CheetahClient clientB;
        private const ushort TurretsParamsFieldId = 333;
        private const ushort DropMineEventId = 555;
        private const ushort HealFieldId = 777; 

         

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
            clientA = ConnectToRelay(ticketA.Result,codecRegistry);
            clientA.AttachToRoom();

            // подключаем второрого клиента
            var ticketB = PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector,"user_b");
            yield return Enumerators.Await(ticketB);
            clientB = ConnectToRelay(ticketB.Result,codecRegistry);
            clientB.AttachToRoom();
            
            // полуаем сетевые команды, которые не надо учитывать в тестах
            yield return new WaitForSeconds(2);
            clientA.Update();
            clientB.Update();
            
        }

        [UnityTest]
        public IEnumerator TestCreatedObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new CreatedObjectIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var objectBuilder = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup);
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            objectBuilder.SetStructure(TurretsParamsFieldId, ref turretsParams);
            var createdObject = objectBuilder.Build();
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var createdObjectStream= collector.GetStream();
            var cheetahObjectConstructor = createdObjectStream[0];
            Assert.AreEqual(createdObject.ObjectId, cheetahObjectConstructor.cheetahObject.ObjectId);
            
            // проверяем структуру
            var incomeTurretsParams = new TurretsParamsStructure();
            cheetahObjectConstructor.GetStruct(TurretsParamsFieldId, ref incomeTurretsParams);
            Assert.AreEqual(turretsParams, incomeTurretsParams);
        }
        
        [UnityTest]
        public IEnumerator TestEventIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new EventIncomeCommandCollector<DropMineEvent>(clientB, DropMineEventId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            createdObject.SendEvent(DropMineEventId, ref dropMineEvent);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream= collector.GetStream();
            Assert.AreEqual(dropMineEvent.MineId, eventsStream.GetItem(0).data.MineId);
        }
        
        [UnityTest]
        public IEnumerator TestStructureIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new StructureIncomeCommandCollector<TurretsParamsStructure>(clientB, TurretsParamsFieldId);
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
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var structuresStream= collector.GetStream();
            var turretsParamsStructure = structuresStream.GetItem(0).data;
            Assert.AreEqual(turretsParams.Damage, turretsParamsStructure.Damage);
            Assert.AreEqual(turretsParams.Speed, turretsParamsStructure.Speed);
        }
        
        [UnityTest]
        public IEnumerator TestLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.SetLong(HealFieldId, 7799);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var value = stream.GetItem(0).data;
            Assert.AreEqual(value, 7799);
        }
        
        [UnityTest]
        public IEnumerator TestDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.SetDouble(HealFieldId, 77.99);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream= collector.GetStream();
            var value = stream.GetItem(0).data;
            Assert.AreEqual(value, 77.99);
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