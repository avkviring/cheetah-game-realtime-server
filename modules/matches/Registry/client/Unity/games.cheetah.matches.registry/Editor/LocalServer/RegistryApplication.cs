using System.Collections.Generic;
using Cheetah.Matches.Relay.Editor.LocalServer.Application;
using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Registry.Editor.LocalServer
{
    public class RegistryApplication : PlatformApplication
    {
        private RelayApplication relayApplication;
        public const string AppName = "cheetah-matches-stub-registry";

        public RegistryApplication() : base(AppName)
        {
            Dependencies.Add(RelayApplication.AppName);
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv("MATCHES_RELAY_INTERNAL_GRPC_HOST", RelayApplication.AppName);
            builder.AddEnv("MATCHES_RELAY_INTERNAL_GRPC_PORT", InternalGrpcPort.ToString());
            builder.AddEnv("MATCHES_RELAY_EXTERNAL_GAME_HOST", relayApplication.Config.Host);
            builder.AddEnv("MATCHES_RELAY_EXTERNAL_GAME_PORT", relayApplication.Config.Port.ToString());
        }

        public override void ConfigureFromApplications(IList<ServerApplication> applications)
        {
            base.ConfigureFromApplications(applications);
            foreach (var application in applications)
            {
                if (application is RelayApplication relay)
                {
                    relayApplication = relay;
                }
            }
        }
    }
}