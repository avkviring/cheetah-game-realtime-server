using System.Collections;
using Cheetah.Platform;
using Cheetah.Platform.Tests;
using Cheetah.User.Accounts.Cookie;
using Cheetah.User.Store;
using NUnit.Framework;
using Tests.Helpers;
using UnityEngine.TestTools;

namespace Tests.User.Store
{
    public class UserStoreTest
    {
        private ClusterConnector _clusterConnector;
        private Cheetah.User.Accounts.User _user;

        [UnitySetUp]
        public IEnumerator SetUp()
        {
            var factory = new KubernetesOrDockerConnectorFactory();
            yield return Enumerators.Await(factory.Connect());
            _clusterConnector = factory.ClusterConnector;

            var cookieAuthenticator = new CookieAuthenticator(_clusterConnector, "test");
            var loginOrRegisterTask = cookieAuthenticator.LoginOrRegister();
            yield return Enumerators.Await(loginOrRegisterTask);
            cookieAuthenticator.RemoveLocalCookie();
            _user = loginOrRegisterTask.Result.User;
        }

        [UnityTest]
        public IEnumerator TestSet()
        {
            var update = new Updater(_clusterConnector, _user);

            string field = "pet";
            string fieldValue = "Archie";

            yield return Enumerators.Await(update.SetString(field, fieldValue));

            var fetch = new Fetcher(_clusterConnector, _user);
            var fetchTask = fetch.TryGetString(field);
            yield return Enumerators.Await(fetchTask);

            var returnedFieldvalue = fetchTask.Result;
            Assert.AreEqual(returnedFieldvalue, fieldValue);
        }

        [UnityTest]
        public IEnumerator TestIncrement()
        {
            var update = new Updater(_clusterConnector, _user);

            string field = "rage_score";
            long fieldValue = 9999;
            long incrementValue = 1;

            yield return Enumerators.Await(update.SetLong(field, fieldValue));
            yield return Enumerators.Await(update.IncrementLong(field, incrementValue));

            var fetch = new Fetcher(_clusterConnector, _user);
            var fetchTask = fetch.TryGetLong(field);
            yield return Enumerators.Await(fetchTask);

            var returnedFieldvalue = fetchTask.Result;
            Assert.AreEqual(returnedFieldvalue, fieldValue + incrementValue);
        }

        [UnityTearDown]
        public IEnumerator TearDown()
        {
            yield return Enumerators.Await(_clusterConnector.Destroy());
        }
    }
}
