using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Games_Cheetah_Client_Tests_Server_Types;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Field;
using Games.Cheetah.EmbeddedServer.API;
using Games.Cheetah.GRPC.Internal;
using NUnit.Framework;
using FieldType = Games.Cheetah.GRPC.Shared.FieldType;

namespace Games.Cheetah.Client.Tests.Server.Helpers
{
    public abstract class AbstractTest
    {
        protected NetworkClient clientA;
        protected NetworkClient clientB;
        private RoomIdResponse roomIdResponse;
        protected CreateMemberResponse memberA;
        protected CreateMemberResponse memberB;
        private EmbeddedServer.API.EmbeddedServer server;
        protected static FieldId.Structure TurretsParamsFieldId = new(100);
        protected static FieldId.Event DropMineEventFieldIdId = new(555);
        protected static FieldId.Double HealFieldId = new(777);
        protected static FieldId.Long ScoreFieldId = new(999);


        [SetUp]
        public void SetUp()
        {
            server = new EmbeddedServer.API.EmbeddedServer(IPAddress.Loopback);
            EmbeddedServer.API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Error);
            var grpcClient = server.CreateGrpcClient();
            Task.Run(async () =>
            {
                roomIdResponse = await grpcClient.CreateRoomAsync(new RoomTemplate
                {
                    Permissions = new Permissions
                    {
                        Objects =
                        {
                            new GameObjectTemplatePermission
                            {
                                Template = 777,
                                Fields =
                                {
                                    new PermissionField
                                    {
                                        Type = FieldType.Long,
                                        Id = ScoreFieldId.Id,
                                        Rules =
                                        {
                                            new GroupsPermissionRule
                                            {
                                                Groups = PlayerHelper.PlayerGroup,
                                                Permission = PermissionLevel.Rw
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
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

        private static NetworkClient ConnectToServer(EmbeddedServer.API.EmbeddedServer server, ulong roomId, CreateMemberResponse member,
            CodecRegistryBuilder codecRegistryBuilder)
        {
            var client = new NetworkClient(server.GetUdpGameHost(), server.GetUdpGamePort(), member.UserId, roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistryBuilder.Build());
            client.DisableClientLog();
            return client;
        }

        [TearDown]
        public void TearDown()
        {
            clientA.Dispose();
            clientB.Dispose();
            server.Destroy();
        }
    }
}