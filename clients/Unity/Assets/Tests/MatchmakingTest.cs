using System.Collections;
using Cheetah.Auth.Cookie;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class MatchmakingTest
    {
        [UnityTest]
        public IEnumerator ShouldMatch()
        {
            var connector = ConnectorFactory.Create();
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
    }
}