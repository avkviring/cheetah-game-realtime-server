using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class ServerTimeTest : AbstractTest
    {
        [Test]
        public void Test()
        {
            // ждем отправки команды
            Thread.Sleep(2000);
            Assert.True(clientA.GetServerTimeInMs() > 1000);
        }
    }
}