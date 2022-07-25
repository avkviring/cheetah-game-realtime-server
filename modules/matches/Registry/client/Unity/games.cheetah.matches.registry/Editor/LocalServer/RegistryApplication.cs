using System.Collections.Generic;
using Cheetah.Matches.Realtime.Editor.LocalServer.Application;
using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Registry.Editor.LocalServer
{
    public class RegistryApplication : PlatformApplication
    {
        private RealtimeApplication realtimeApplication;
        public const string AppName = "matches-stubregistry";

        public RegistryApplication() : base(AppName)
        {
            Dependencies.Add(RealtimeApplication.AppName);
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv("MATCHES_RELAY_INTERNAL_GRPC_HOST", RealtimeApplication.AppName);
            builder.AddEnv("MATCHES_RELAY_INTERNAL_GRPC_PORT", InternalGrpcPort.ToString());
            builder.AddEnv("MATCHES_RELAY_EXTERNAL_GAME_HOST", realtimeApplication.Config.Host);
            builder.AddEnv("MATCHES_RELAY_EXTERNAL_GAME_PORT", realtimeApplication.Config.Port.ToString());
        }

        public override void ConfigureFromApplications(IList<ServerApplication> applications)
        {
            base.ConfigureFromApplications(applications);
            foreach (var application in applications)
            {
                if (application is RealtimeApplication relay)
                {
                    realtimeApplication = relay;
                }
            }
        }
    }
}