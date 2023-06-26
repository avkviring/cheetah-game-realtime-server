using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using Games.Cheetah.Client.Types.Network;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class ReconnectTest : AbstractTest
    {
        [Test]
        public void ShouldReconnect()
        {
            // загружаем объекты комнаты - они нам не интересны
            clientA.Update();
            Thread.Sleep(200);

            // создаем объект на первом клиенте

            clientA.NewObjectBuilder(55, PlayerHelper.PlayerGroup).Build();
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientA.Update();


            var newClientA = new NetworkClient(2, clientA.serverUdpHost, clientA.serverUdpPort, clientA.MemberId, clientA.roomId,
                clientA.privateUserKey,
                clientA.CodecRegistry);


            newClientA.AttachToRoom();
            Thread.Sleep(500);
            newClientA.Update();
            Assert.AreEqual(newClientA.GetConnectionStatus(), ConnectionStatus.Connected);
            
            var objectsClientA = newClientA.Reader.GetCreatedObjectsInCurrentUpdate(55);
            Assert.AreEqual(objectsClientA.Count, 1);
        }
    }
}