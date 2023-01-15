using System;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class MemberConnect : AbstractTest
    {
        [Test]
        public void ShouldMemberConnectCommand()
        {
            ConnectToServer(server, roomIdResponse.RoomId, memberC, codecRegistry);
            Thread.Sleep(TimeSpan.FromMilliseconds(200));
            clientA.Update();
            var connected = clientA.Reader.GetConnectedMemberInUpdate();
            Assert.AreEqual(connected.Length, 1);
            Assert.AreEqual(connected[0], memberC.UserId);
            connected.Dispose();
        }

        [Test]
        public void ShouldMemberDisconnectCommand()
        {
            var client = ConnectToServer(server, roomIdResponse.RoomId, memberC, codecRegistry);
            Thread.Sleep(TimeSpan.FromMilliseconds(200));
            client.Dispose();
            Thread.Sleep(TimeSpan.FromMilliseconds(500));
            clientA.Update();
            var disconnected = clientA.Reader.GetDisconnectedMemberInUpdate();
            Assert.AreEqual(disconnected.Length, 1);
            Assert.AreEqual(disconnected[0], memberC.UserId);
            disconnected.Dispose();
        }
    }
}