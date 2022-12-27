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
            Thread.Sleep(TimeSpan.FromMilliseconds(1000));
            clientA.Update();
            var connected = clientA.Reader.GetConnectedMemberInUpdate();
            Assert.AreEqual(connected.Length, 1);
            Assert.AreEqual(connected[0], memberC.UserId);
        }
    }
}