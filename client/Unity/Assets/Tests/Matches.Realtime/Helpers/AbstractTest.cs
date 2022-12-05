using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.GRPC.Internal;
using NUnit.Framework;
using Shared;
using Shared_Types;

namespace Tests.Matches.Realtime.Helpers
{
    public abstract class AbstractTest
    {
        protected CheetahClient clientA;

        protected CheetahClient clientB;
        private RoomIdResponse roomIdResponse;
        protected CreateMemberResponse memberA;
        protected CreateMemberResponse memberB;
        private EmbeddedServer server;
        protected const ushort TurretsParamsFieldId = 333;
        protected const ushort DropMineEventId = 555;
        protected const ushort HealFieldId = 777;

        [SetUp]
        public void SetUp()
        {
            server = new EmbeddedServer(IPAddress.Loopback);
            var grpcClient = server.CreateGrpcClient();
            Task.Run(async () =>
            {
                roomIdResponse = await grpcClient.CreateRoomAsync(new RoomTemplate());
                memberA = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = roomIdResponse.RoomId,
                    User = new UserTemplate
                    {
                        Groups = PlayerHelper.PlayerGroup
                    }
                });
                memberB = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = roomIdResponse.RoomId,
                    User = new UserTemplate
                    {
                        Groups = PlayerHelper.PlayerGroup
                    }
                });
            }).GetAwaiter().GetResult();


            var codecRegistry = new CodecRegistryBuilder();
            codecRegistry.Register(_ => new GlobalNamespaceObjectCodec());
            codecRegistry.Register(_ => new DropMineEventCodec());
            codecRegistry.Register(_ => new SomeSingletonKeyCodec());
            codecRegistry.Register(_ => new TurretsParamsStructureCodec());


            // подключаем первого клиента
            clientA = ConnectToServer(server, roomIdResponse.RoomId, memberA, codecRegistry);
            clientA.AttachToRoom();

            // подключаем второго клиента
            clientB = ConnectToServer(server, roomIdResponse.RoomId, memberB, codecRegistry);
            clientB.AttachToRoom();

            // полуаем сетевые команды, которые не надо учитывать в тестах
            Thread.Sleep(200);
            clientA.Update();
            clientB.Update();
        }

        private static CheetahClient ConnectToServer(EmbeddedServer server, ulong roomId, CreateMemberResponse member,
            CodecRegistryBuilder codecRegistryBuilder)
        {
            var client = new CheetahClient(server.GetGameUri(), member.UserId, roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistryBuilder.Build());
            client.DisableClientLog();
            return client;
        }

        [TearDown]
        public void TearDown()
        {
            clientA.Destroy();
            clientB.Destroy();
            server.Destroy();
        }
    }
}