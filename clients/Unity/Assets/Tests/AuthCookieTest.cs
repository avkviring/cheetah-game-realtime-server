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
        private GRPCConnector grpcConnector;


        [UnityTest]
        public IEnumerator ShouldCreateUser()
        {
            var connectorFactory = new ConnectorFactory();
            yield return Enumerators.Await(connectorFactory.Connect());
            grpcConnector = connectorFactory.GrpcConnector;

            var cookieAuthenticator = new CookieAuthenticator(grpcConnector);
            cookieAuthenticator.RemoveLocalCookie();
            yield return Enumerators.Await(cookieAuthenticator.LoginOrRegister());
        }

        [TearDown]
        public async void TearDown()
        {
            await grpcConnector.Destroy();
        }
    }
}