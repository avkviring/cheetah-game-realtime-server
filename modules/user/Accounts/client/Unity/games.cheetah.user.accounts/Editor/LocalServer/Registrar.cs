using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using UnityEditor;

namespace Cheetah.Accounts.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("accounts", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            var redis = new RedisApplication(AccountsApplication.AppName);
            var service = new AccountsApplication(redis.Name);
            IList<ServerApplication> result = new List<ServerApplication>();
            result.Add(service);
            result.Add(redis);
            return result;
        }
    }
}