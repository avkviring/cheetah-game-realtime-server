using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;
using UnityEngine;

namespace Games.Cheetah.Client.Tests.Server
{
    public class StatisticsTest : AbstractTest
    {
        public void ShouldStatistics()
        {
            clientA.AttachToRoom();
            clientB.AttachToRoom();

            var prevStatisticsA = clientA.GetStatistics();
            var prevStatisticsB = clientB.GetStatistics();
            clientA.NewObjectBuilder(1, 8).Build();
            // ждем отправки команды
            Thread.Sleep(200);
            var currentStatisticsA = clientA.GetStatistics();
            var currentStatisticsB = clientB.GetStatistics();

            Debug.Log(currentStatisticsA);
            Debug.Log(currentStatisticsB);
            
            // грубая проверка, так как есть служебные пакеты и мы не знаем когда они будут отправлены
            Assert.True(currentStatisticsA.LastFrameId > prevStatisticsA.LastFrameId);
            Assert.True(currentStatisticsA.SendSize > prevStatisticsA.SendSize);
            Assert.True(currentStatisticsA.SendPacketCount > prevStatisticsA.SendPacketCount);

            Assert.True(currentStatisticsB.LastFrameId > prevStatisticsB.LastFrameId);
            Assert.True(currentStatisticsB.ReceiveSize > prevStatisticsB.ReceiveSize);
            Assert.True(currentStatisticsB.ReceivePacketCount > prevStatisticsB.ReceivePacketCount);
        }
    }
}