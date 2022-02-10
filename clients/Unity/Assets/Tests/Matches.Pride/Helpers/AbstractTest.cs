using System.Collections;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Platform;
using NUnit.Framework;
using Shared;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Pride.Helpers
{
    public abstract class AbstractTest
    {
        protected ClusterConnector clusterConnector;
        protected CheetahClient clientA;

        protected CheetahClient clientB;
        protected uint memberA;
        protected uint memberB;
        protected const ushort TurretsParamsFieldId = 333;
        protected const ushort DropMineEventId = 555;
        protected const ushort HealFieldId = 777;

        [UnitySetUp]
        public IEnumerator SetUp()
        {
            var codecRegistry = new CodecRegistryBuilder();
            // codecRegistry.RegisterEventCodec(DropMineEventId, new DropMineEventCodec());
            // codecRegistry.RegisterStructureCodec(TurretsParamsFieldId, new TurretsParamsStructureCodec());

            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            // подключаем первого клиента
            var ticketA = UserHelper.CreateNewPlayerAndMatchToBattle(clusterConnector, "user_a");
            yield return Enumerators.Await(ticketA);
            memberA = ticketA.Result.UserId;
            clientA = ConnectToRelay(ticketA.Result, codecRegistry);
            clientA.AttachToRoom();

            // подключаем второрого клиента
            var ticketB = UserHelper.CreateNewPlayerAndMatchToBattle(clusterConnector, "user_b");
            yield return Enumerators.Await(ticketB);
            memberB = ticketB.Result.UserId;
            clientB = ConnectToRelay(ticketB.Result, codecRegistry);
            clientB.AttachToRoom();

            // полуаем сетевые команды, которые не надо учитывать в тестах
            yield return new WaitForSeconds(1);
            clientA.Update();
            clientB.Update();
        }

        private static CheetahClient ConnectToRelay(TicketResponse ticket, CodecRegistryBuilder codecRegistryBuilder)
        {
            return new CheetahClient(ticket.RelayGameHost, ticket.RelayGamePort, ticket.UserId, ticket.RoomId, ticket.PrivateKey.ToByteArray(),
                codecRegistryBuilder.Build());
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