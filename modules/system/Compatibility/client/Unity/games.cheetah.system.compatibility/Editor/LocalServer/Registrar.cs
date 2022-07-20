using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.System.Compatibility.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("system.compatibility", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            return new List<ServerApplication>()
            {
                new CompatibilityApplication()
            };
        }
    }
}