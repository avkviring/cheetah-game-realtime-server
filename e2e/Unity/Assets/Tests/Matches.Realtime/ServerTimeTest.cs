using System.Collections;
using NUnit.Framework;
using Tests.Matches.Realtime.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests.Matches.Realtime
{
    public class ServerTimeTest : AbstractTest
    {
        [UnityTest]
        public IEnumerator Test()
        {
            // ждем отправки команды
            yield return new WaitForSeconds(2);
            Assert.True(clientA.GetServerTimeInMs() > 1000);
        }
    }
}