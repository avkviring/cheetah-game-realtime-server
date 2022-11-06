using System.Net;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.EmbeddedServer.Test
{
    public class EmbeddedServerTests
    {
        [Test]
        public void Test()
        {
            API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);
            var server = new API.EmbeddedServer(IPAddress.Any);
            var room = server.CreateRoom();
            var member = room.CreateMember(0b000111);
            API.EmbeddedServer.ShowCurrentLogs();
        }
    }
}