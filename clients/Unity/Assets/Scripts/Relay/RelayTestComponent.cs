using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Matches.Relay;
using Cheetah.Matches.Relay.Codec;
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
        private CheetahObject objectA;
        private CheetahObject objectB;
        private CheetahClient relayClient;
        private ClusterConnector clusterConnector;

        private async void OnEnable()
        {
            clusterConnector = new ClusterConnector("127.0.0.1", 7777, false);
            var ticket = await PlayerHelper.CreateNewPlayerAndMatchToBattle(clusterConnector,"user");
            ConnectToRelay(ticket);
            CreateRelayObjects();
        }

        private void ConnectToRelay(TicketResponse ticket)
        {
            relayClient = new CheetahClient(ticket.RelayGameHost, ticket.RelayGamePort, ticket.UserId, ticket.RoomId, ticket.PrivateKey.ToByteArray(),
                new CodecRegistry());
        }

        private void CreateRelayObjects()
        {
            objectA = relayClient.NewObjectBuilder(1, PlayerHelper.UserGroup).Build();
            objectB = relayClient.NewObjectBuilder(100, PlayerHelper.UserGroup).Build();
        }

        private long counter;


        private void Update()
        {
            objectA.IncrementLong(2, counter);
            objectB.IncrementDouble(20, counter);
            objectB.IncrementDouble(30, 10);
            relayClient.Update();
            counter++;
        }

        private async void OnDestroy()
        {
            relayClient.Destroy();
            await clusterConnector.Destroy();
        }
    }
}