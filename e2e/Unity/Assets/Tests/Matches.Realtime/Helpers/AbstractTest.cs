using System.Net;
using System.Threading;
using Cheetah.Matches.Realtime;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using NUnit.Framework;
using Shared;
using Shared_Types;

namespace Tests.Matches.Realtime.Helpers
{
    public abstract class AbstractTest
    {
        protected CheetahClient clientA;

        protected CheetahClient clientB;
        protected ServerMember memberA;
        protected ServerMember memberB;
        private EmbeddedServer server;
        protected const ushort TurretsParamsFieldId = 333;
        protected const ushort DropMineEventId = 555;
        protected const ushort HealFieldId = 777;

        [SetUp]
        public void SetUp()
        {
            server = new EmbeddedServer(IPAddress.Loopback);
            var room = server.CreateRoom();

            memberA = room.CreateMember(PlayerHelper.PlayerGroup);
            memberB = room.CreateMember(PlayerHelper.PlayerGroup);


            var codecRegistry = new CodecRegistryBuilder();
            codecRegistry.Register(factory=>new GlobalNamespaceObjectCodec());
            codecRegistry.Register(factory=>new DropMineEventCodec());
            codecRegistry.Register(factory=>new SomeSingletonKeyCodec());
            codecRegistry.Register(factory=>new TurretsParamsStructureCodec());


            // подключаем первого клиента
            clientA = ConnectToRelay(server, room, memberA, codecRegistry);
            clientA.AttachToRoom();

            // подключаем второго клиента
            clientB = ConnectToRelay(server, room, memberB, codecRegistry);
            clientB.AttachToRoom();

            // полуаем сетевые команды, которые не надо учитывать в тестах
            Thread.Sleep(200);
            clientA.Update();
            clientB.Update();
        }

        private static CheetahClient ConnectToRelay(EmbeddedServer server, ServerRoom room, ServerMember member,
            CodecRegistryBuilder codecRegistryBuilder)
        {
            var client = new CheetahClient(server.GetGameUri(), member.GetId(), room.GetId(),
                member.GetPrivateKey(),
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