using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay.Command;
using Cheetah.Platform;
using Shared;
using UnityEngine;

namespace Relay
{
    /// <summary>
    /// Запуск "игры" на relay сервер
    /// </summary>
    public class RelayTestComponent : MonoBehaviour
    {
        private ushort clientId;
        private CheetahObjectId objectA;
        private CheetahObjectId objectB;
        private ClusterConnector clusterConnector;

        private async void OnEnable()
        {
            clusterConnector = new ClusterConnector("127.0.0.1", 7777, false);
            var ticket = await PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector);
            ConnectToRelay(ticket);
            CreateRelayObjects();
        }

        private void ConnectToRelay(TicketResponse ticket)
        {
            var userPrivateKey = new CheetahBuffer(ticket.PrivateKey.ToByteArray());
            CheetahClient.CreateClient(ticket.RelayGameHost + ":" + ticket.RelayGamePort, (ushort)ticket.UserId, ticket.RoomId,
                ref userPrivateKey, 0, out clientId);
            CheetahClient.SetCurrentClient(clientId);
        }

        private void CreateRelayObjects()
        {
            CheetahObject.Create(1, PlayerHelper.UserGroup, ref objectA);
            CheetahObject.Created(ref objectA);
            CheetahObject.Create(100, PlayerHelper.UserGroup, ref objectB);
            CheetahObject.Created(ref objectB);
        }

        private long counter;

        private void Update()
        {
            if (clientId == 0)
            {
                return;
            }

            CheetahLong.Increment(ref objectA, 2, counter);
            CheetahDouble.Increment(ref objectB, 20, counter);
            CheetahDouble.Increment(ref objectB, 30, 10);
            CheetahClient.Receive();
            counter++;
        }

        private async void OnDestroy()
        {
            await clusterConnector.Destroy();
        }
    }
}