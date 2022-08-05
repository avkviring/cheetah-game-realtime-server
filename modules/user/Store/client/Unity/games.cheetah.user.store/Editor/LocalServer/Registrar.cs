using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using UnityEditor;

namespace Cheetah.User.Accounts.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("userstore", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            var userstore = new UserStoreApplication();
            var apps = new List<ServerApplication>() { userstore };

            return apps;
        }
    }
}
