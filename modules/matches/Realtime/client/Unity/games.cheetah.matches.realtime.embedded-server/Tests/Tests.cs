using System;
using System.Threading;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.EmbeddedServer.API;
using Cheetah.Matches.Realtime.Logger;
using Cheetah.Matches.Realtime.Types;
using NUnit.Framework;

namespace Cheetah.Matches.Realtime.EmbeddedServer.Tests
{
    public class EmbeddedServerTests
    {
        [Test]
        public void Test()
        {
            API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);
            
            var server = new API.EmbeddedServer();
            var room = server.CreateRoom();
            var member = room.CreateMember(0b000111);

            var client = new CheetahClient(
                server.GetGameHost(),
                server.GetGamePort(),
                member.GetId(),
                room.GetId(),
                member.GetPrivateKey(),
                new CodecRegistryBuilder().Build());
            client.DisableClientLog();
            client.Update();

            // небольшая пауза для обмена сетевыми пакетами
            Thread.Sleep(TimeSpan.FromSeconds(1));

            // проверяем факт соединения
            Assert.AreEqual(client.GetConnectionStatus(), CheetahClientConnectionStatus.Connected);

            // останавливаем сервер
            server.Destroy();

            // сервер остановлен - выжидаем окончания timeout на клиентские команды
            Thread.Sleep(TimeSpan.FromSeconds(11));
            Assert.AreNotEqual(client.GetConnectionStatus(), CheetahClientConnectionStatus.Connected);
            
            API.EmbeddedServer.ShowCurrentLogs();
        }
    }
}