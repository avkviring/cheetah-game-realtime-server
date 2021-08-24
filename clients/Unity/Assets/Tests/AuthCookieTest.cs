using System.Collections;
using Cheetah.Auth.Cookie;
using Cheetah.Platform;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine;
using UnityEngine.TestTools;

namespace Tests
{
    public class AuthCookieTest
    {
        private Connector connector;

        [SetUp]
        public void SetUp()
        {
            connector = ConnectorFactory.Create();
        }

        [UnityTest]
        public IEnumerator ShouldCreateUser()
        {
            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();
            yield return Enumerators.Await(cookieAuthenticator.LoginOrRegister());
        }

        [TearDown]
        public async void TearDown()
        {
            await connector.Shutdown();
        }
    }
}