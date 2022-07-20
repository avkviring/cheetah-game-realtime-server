using System.Collections.Generic;
using Cheetah.Matches.Factory.Editor.LocalServer.Factory;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.Matches.Factory.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("matches.factory", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            return new List<ServerApplication>()
            {
                new FactoryApplication(),
            };
        }
    }
}