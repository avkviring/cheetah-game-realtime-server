using System.Collections;
using Cheetah.Auth.Cookie;
using Cheetah.Matches.Matchmaking;
using Cheetah.Platform;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class MatchmakingTest
    {
        private Connector connector;


        [UnityTest]
        public IEnumerator ShouldMatch()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            connector = connectorFactory.Connector;

            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();

            var loginOrRegisterTask = cookieAuthenticator.LoginOrRegister();
            yield return Enumerators.Await(loginOrRegisterTask);

            var createPlayerResult = loginOrRegisterTask.Result;
            var scheduleUserToMatchTask = MatchmakingScheduler.ScheduleUserToMatch(createPlayerResult.Player, "gubaha", 256);
            yield return Enumerators.Await(scheduleUserToMatchTask);

            var matchmakingResult = scheduleUserToMatchTask.Result;
            Debug.Log(matchmakingResult.RelayGameHost);
            Assert.IsTrue(true);
        }

        [TearDown]
        public async void TearDown()
        {
            await connector.Shutdown();
        }
    }
}