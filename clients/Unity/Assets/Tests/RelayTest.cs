using System.Collections;
using AOT;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay.Command;
using Cheetah.Platform;
using NUnit.Framework;
using Shared;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class MatchmakingTest
    {
        private ClusterConnector clusterConnector;
        private static CheetahObjectId? createdObjectId;


        [UnityTest]
        public IEnumerator ShouldEnterToRelay()
        {
            createdObjectId = null;

            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            // первый клиент создает объект на relay сервере
            var playerATicketTask = PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector);
            yield return Enumerators.Await(playerATicketTask);
            var ticketUserA = playerATicketTask.Result;
            var relayClientA = ConnectToRelay(ticketUserA);
            CheetahClient.SetCurrentClient(relayClientA);
            var objectId = new CheetahObjectId();
            CheetahObject.Create(777, PlayerHelper.UserGroup, ref objectId);
            CheetahObject.Created(ref objectId);

            // второй клиент должен загрузить созданный объект
            var playerBTicketTask = PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector);
            yield return Enumerators.Await(playerBTicketTask);
            var ticketUserB = playerBTicketTask.Result;
            var relayClientB = ConnectToRelay(ticketUserB);
            CheetahClient.SetCurrentClient(relayClientB);
            CheetahObject.SetCreatedListener(CreatedListener);
            CheetahClient.AttachToRoom();
            yield return new WaitForSeconds(3);
            CheetahClient.Receive();

            Assert.AreEqual(objectId, createdObjectId);
        }

        [MonoPInvokeCallback(typeof(CheetahObject.CreatedListener))]
        private static void CreatedListener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId)
        {
            if (objectId.roomOwner)
            {
                return;
            }

            createdObjectId = objectId;
        }


        private ushort ConnectToRelay(TicketResponse ticket)
        {
            var userPrivateKey = new CheetahBuffer(ticket.PrivateKey.ToByteArray());
            CheetahClient.CreateClient(ticket.RelayGameHost + ":" + ticket.RelayGamePort, (ushort)ticket.UserId, ticket.RoomId, ref userPrivateKey, 0,
                out var clientId);
            return clientId;
        }

        [TearDown]
        public async void TearDown()
        {
            await clusterConnector.Destroy();
        }
    }
}