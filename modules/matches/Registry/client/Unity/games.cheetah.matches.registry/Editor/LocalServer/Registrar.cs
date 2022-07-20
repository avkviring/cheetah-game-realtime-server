using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.Matches.Registry.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Platform.Editor.LocalServer.Registry.Register("matches.registry", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            return new List<ServerApplication>()
            {
                new RegistryApplication(),
            };
        }
    }
}