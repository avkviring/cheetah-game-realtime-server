using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer;
using UnityEditor;

namespace Cheetah.Matches.Matchmaking.Editor.LocalServer
{
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            Registry.Register("matches.matchmaking", CreateApplications());
        }

        private static IList<ServerApplication> CreateApplications()
        {
            return new List<ServerApplication>()
            {
                new MatchmakingApplication()
            };
        }
    }
}