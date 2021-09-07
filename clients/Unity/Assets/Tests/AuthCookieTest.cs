using System.Collections;
using Cheetah.Auth.Cookie;
using Cheetah.Platform;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine.TestTools;

namespace Tests
{
    public class AuthCookieTest
    {
        private Connector connector;


        [UnityTest]
        public IEnumerator ShouldCreateUser()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            connector = connectorFactory.Connector;

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