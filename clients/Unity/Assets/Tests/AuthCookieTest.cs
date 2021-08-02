using Cheetah.Auth.Cookie;
using Cheetah.Platform;
using NUnit.Framework;
using UnityEngine;

namespace Tests
{
    public class AuthCookieTest
    {
        [Test]
        public async void ShouldCreateUser()
        {
            Connector connector = ConnectorFactory.Create();
            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();
            var result = await cookieAuthenticator.LoginOrRegister();
            Debug.Log(result.Player.SessionToken);
            Assert.IsTrue(true);
        }
    }
}