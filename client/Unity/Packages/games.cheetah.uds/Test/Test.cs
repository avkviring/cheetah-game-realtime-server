using System;
using System.Net;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.GRPC.Internal;
using Cheetah.Matches.Realtime.Types;
using Games.Cheetah.UDS.API;
using NUnit.Framework;

namespace Games.Cheetah.UDS.Tests
{
    public class Test
    {
        private ulong createdRoomId;
        private CheetahClient cheetahClient;
        private EmbeddedServer server;
        private UDSPlugin plugin;


        [SetUp]
        public void SetUp()
        {
            EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);
            server = new EmbeddedServer(IPAddress.Loopback);
            plugin = new UDSPlugin(
                server.GetInternalGrpcUri(),
                server.GetInternalWebGrpcUri(),
                server.GetGameUri(),
                OnRoomCreated,
                OnRoomDeleted,
                new CodecRegistryBuilder().Build());
        }


        [Test]
        public void ShouldCreatePluginWhenCreateRoom()
        {
            Task.Run(async () =>
            {
                var grpcClient = server.CreateGrpcClient();
                var room = await grpcClient.CreateRoomAsync(new RoomTemplate());
                await Task.Delay(TimeSpan.FromSeconds(2));

                await plugin.OnUpdate();
                // даем время cheetahClient перейти в состояние connected
                await Task.Delay(TimeSpan.FromSeconds(2));

                Assert.AreEqual(room.RoomId, createdRoomId);
                Assert.AreEqual(cheetahClient.GetConnectionStatus(), CheetahClientConnectionStatus.Connected);
            }).GetAwaiter().GetResult();
        }


        private void OnRoomDeleted(ulong roomId)
        {
            throw new NotImplementedException();
        }

        private void OnRoomCreated(ulong roomId, Realtime.RealtimeClient grpClient, CheetahClient cheetahClient)
        {
            createdRoomId = roomId;
            this.cheetahClient = cheetahClient;
        }
    }
}