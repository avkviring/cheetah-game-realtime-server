namespace Tests
{
    public class MatchmakingTest
    {
        // private Connector connector;
        //
        // [UnitySetUp, Timeout(500_000)]
        // public IEnumerator SetUp()
        // {
        //     var connectorFactory = new ConnectorFactory();
        //     yield return connectorFactory.Connect();
        //     connector = connectorFactory.Connector;
        // }
        //
        // [UnityTest]
        // public IEnumerator ShouldMatch()
        // {
        //     var cookieAuthenticator = new CookieAuthenticator(connector);
        //     cookieAuthenticator.RemoveLocalCookie();
        //
        //     var loginOrRegisterTask = cookieAuthenticator.LoginOrRegister();
        //     yield return Enumerators.Await(loginOrRegisterTask);
        //
        //     var createPlayerResult = loginOrRegisterTask.Result;
        //     var scheduleUserToMatchTask = MatchmakingScheduler.ScheduleUserToMatch(createPlayerResult.Player, "/gubaha", new UserTemplate());
        //     yield return Enumerators.Await(scheduleUserToMatchTask);
        //
        //     var matchmakingResult = scheduleUserToMatchTask.Result;
        //     Debug.Log(matchmakingResult.RelayGameHost);
        //     Assert.IsTrue(true);
        // }
        //
        // [TearDown]
        // public async void TearDown()
        // {
        //     await connector.Shutdown();
        // }
    }
}