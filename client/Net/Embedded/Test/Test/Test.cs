using System.Net;
using Games.Cheetah.EmbeddedServer.API;
using Games.Cheetah.GRPC.Internal;
using NUnit.Framework;

namespace Games.Cheetah.EmbeddedServer.Test
{
    public class EmbeddedServerTests
    {
        [Test]
        public void Test()
        {
            API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);
            var server = new API.EmbeddedServer(IPAddress.Any);
            var client = server.CreateGrpcClient();
            var room = client.CreateRoom(new RoomTemplate());
            client.CreateMember(new CreateMemberRequest
            {
                RoomId = room.RoomId,
                User = new UserTemplate
                {
                    Groups = 0b000111
                }
            });
            API.EmbeddedServer.ShowCurrentLogs();
        }
    }
}