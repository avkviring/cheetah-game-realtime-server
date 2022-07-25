using System.Collections.Generic;
using Cheetah.Matches.Realtime.Editor.LocalServer.Application;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.Matches.Realtime.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            var configuration = new RelayApplicationsConfigurator();
            Registry.Register(
                "relay",
                new List<ServerApplication>
                {
                    new RealtimeApplication(configuration)
                },
                new List<IApplicationsConfigurator> {configuration}
            );
        }
    }
}