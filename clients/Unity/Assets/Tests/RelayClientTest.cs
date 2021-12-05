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
        private uint memberA;
        private uint memberB;
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
        public IEnumerator TestDeleteObjectIncomeCommands()
        {
            // слушаем создание новых объектов на втором клиенте
            var collector = new DeleteObjectIncomeCommands(clientB, 777);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            createdObject.Delete();
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            // прием команды
            clientB.Update();
            // проверяем результат
            var deletedObjectStream= collector.GetStream();
            Assert.AreEqual(createdObject, deletedObjectStream.GetItem(0));
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
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestTargetEventIncomeCommands()
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
            createdObject.SendEvent(DropMineEventId, memberB, ref dropMineEvent);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
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
            var actual = stream.GetItem(0);
            Assert.AreEqual( 7799, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
            
        [UnityTest]
        public IEnumerator TestIncrementLongIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new LongIncomeCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.IncrementLong(HealFieldId, 1001);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
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
            var collector = new LongIncomeCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.CompareAndSet(HealFieldId, 0,555,0);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
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
            var actual = stream.GetItem(0);
            Assert.AreEqual( 77.99, actual.value);
            Assert.AreEqual(memberA, actual.commandCreator);
        }
        
        [UnityTest]
        public IEnumerator TestIncrementDoubleIncomeCommands()
        {
            // слушаем события определенного типа
            var collector = new DoubleIncomeCommandCollector(clientB, HealFieldId);
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.UserGroup).Build();
            // изменяем значение
            createdObject.IncrementDouble(HealFieldId, 77.99);
            // ждем отправки команды
            yield return new WaitForSeconds(2);
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