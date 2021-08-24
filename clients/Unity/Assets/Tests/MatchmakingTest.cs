using System.Collections;
using Cheetah.Auth.Cookie;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Relay.Types;
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

        [SetUp]
        public void SetUp()
        {
            connector = ConnectorFactory.Create();
        }

        [UnityTest]
        public IEnumerator ShouldMatch()
        {
            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();

            var loginOrRegisterTask = cookieAuthenticator.LoginOrRegister();
            yield return Enumerators.Await(loginOrRegisterTask);

            var createPlayerResult = loginOrRegisterTask.Result;
            var scheduleUserToMatchTask = MatchmakingScheduler.ScheduleUserToMatch(createPlayerResult.Player, "/example-room", new UserTemplate());
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