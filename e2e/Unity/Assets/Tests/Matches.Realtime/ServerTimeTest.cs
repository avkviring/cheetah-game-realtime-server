using System.Collections;
using System.Threading;
using NUnit.Framework;
using Tests.Matches.Realtime.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime
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