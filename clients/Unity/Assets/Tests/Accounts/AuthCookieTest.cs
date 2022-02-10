using System.Collections;
using Cheetah.Accounts.Cookie;
using Cheetah.Platform;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine.TestTools;

namespace Tests.Accounts
{
    public class AuthCookieTest
    {
        private ClusterConnector clusterConnector;


        [UnityTest]
        public IEnumerator ShouldCreateUser()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            clusterConnector = connectorFactory.ClusterConnector;

            var cookieAuthenticator = new CookieAuthenticator(clusterConnector);
            cookieAuthenticator.RemoveLocalCookie();
            yield return Enumerators.Await(cookieAuthenticator.LoginOrRegister());
        }

        [TearDown]
        public async void TearDown()
        {
            await clusterConnector.Destroy();
        }
    }
}