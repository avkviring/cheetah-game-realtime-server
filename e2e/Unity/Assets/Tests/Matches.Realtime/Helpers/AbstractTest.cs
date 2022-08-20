using System.Collections;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Realtime;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Platform;
using NUnit.Framework;
using Shared;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime.Helpers
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
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            // подключаем первого клиента
            var ticketA = UserHelper.CreateNewPlayerAndMatchToBattle(clusterConnector, "user_a");
            yield return Enumerators.Await(ticketA);
            memberA = ticketA.Result.MemberId;
            clientA = ConnectToRelay(ticketA.Result, codecRegistry);
            clientA.AttachToRoom();

            // подключаем второго клиента
            var ticketB = UserHelper.CreateNewPlayerAndMatchToBattle(clusterConnector, "user_b");
            yield return Enumerators.Await(ticketB);
            memberB = ticketB.Result.MemberId;
            clientB = ConnectToRelay(ticketB.Result, codecRegistry);
            clientB.AttachToRoom();

            // полуаем сетевые команды, которые не надо учитывать в тестах
            yield return new WaitForSeconds(1);
            clientA.Update();
            clientB.Update();
        }

        private static CheetahClient ConnectToRelay(TicketResponse ticket, CodecRegistryBuilder codecRegistryBuilder)
        {
            var client = new CheetahClient(ticket.RealtimeServerHost, ticket.RealtimeServerPort, ticket.MemberId, ticket.RoomId, ticket.PrivateKey.ToByteArray(),
                codecRegistryBuilder.Build());
            client.DisableClientLog();
            return client;
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