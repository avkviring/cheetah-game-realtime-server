using Cheetah.Auth.Cookie;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Relay.Types;
using NUnit.Framework;
using UnityEngine;

namespace Tests
{
    public class MatchmakingTest
    {
        [Test]
        public async void ShouldMatch()
        {
            var connector = ConnectorFactory.Create();
            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();
            var createPlayerResult = await cookieAuthenticator.LoginOrRegister();
            var matchmakingResult = await MatchmakingScheduler.ScheduleUserToMatch(createPlayerResult.Player, "/example-room", new UserTemplate());
            Debug.Log(matchmakingResult.RelayGameHost);
            Assert.IsTrue(true);
        }
    }
}