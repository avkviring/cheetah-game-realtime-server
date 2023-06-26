using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Games_Cheetah_Client_Tests_Server_Types;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Logger;
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
        protected RoomIdResponse roomIdResponse;
        protected CreateMemberResponse memberA;
        protected CreateMemberResponse memberB;
        protected EmbeddedServer.API.EmbeddedServer server;
        protected static FieldId.Structure TurretsParamsFieldId = new(100);
        protected static FieldId.Event DropMineEventFieldIdId = new(555);
        protected static FieldId.Double HealFieldId = new(777);
        protected static FieldId.Long ScoreFieldId = new(999);
        protected CreateMemberResponse memberC;
        protected CodecRegistry codecRegistry;

        public ulong RoomId { get; private set; }


        [SetUp]
        public void SetUp()
        {
            server = new EmbeddedServer.API.EmbeddedServer(IPAddress.Loopback);
            EmbeddedServer.API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Info);
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

                memberC = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = roomIdResponse.RoomId,
                    User = new UserTemplate
                    {
                        Groups = PlayerHelper.PlayerGroup
                    }
                });
            }).GetAwaiter().GetResult();


            codecRegistry =
                new CodecRegistryBuilder()
                    .Register(_ => new GlobalNamespaceObjectCodec())
                    .Register(_ => new DropMineEventCodec())
                    .Register(_ => new SomeSingletonKeyCodec())
                    .Register(_ => new TurretsParamsStructureCodec()).Build();


            RoomId = roomIdResponse.RoomId;
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

        protected static NetworkClient ConnectToServer(EmbeddedServer.API.EmbeddedServer server, ulong roomId, CreateMemberResponse member,
            CodecRegistry codecRegistry)
        {
            var client = new NetworkClient(0, server.GetUdpGameHost(), server.GetUdpGamePort(), member.UserId, roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistry);
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