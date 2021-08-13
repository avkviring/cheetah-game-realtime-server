using System.Collections;
using Cheetah.Auth.Cookie;
using UnityEngine.TestTools;

namespace Tests
{
    public class AuthCookieTest
    {
        [UnityTest]
        public IEnumerator ShouldCreateUser()
        {
            var connector = ConnectorFactory.Create();
            var cookieAuthenticator = new CookieAuthenticator(connector);
            cookieAuthenticator.RemoveLocalCookie();
            yield return Enumerators.Await(cookieAuthenticator.LoginOrRegister());
        }
    }
}