using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.User.Store.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("user.store", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            var userstore = new UserStoreApplication();
            var apps = new List<ServerApplication>() { userstore };
            return apps;
        }
    }
}