using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Factory.Editor.LocalServer.Factory
{
    public class FactoryApplication : PlatformApplication
    {
        public const string AppName = "matches-factory";
        private readonly string roomsConfigPath = ConfigurationPaths.MakeHostedDataPath("matches-factory/rooms/");

        public FactoryApplication() : base(AppName)
        {
            Dependencies.Add("matches-stubregistry");
            AdminGrpcServices.Add("cheetah.matches.factory.admin");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv("TEMPLATES_PATH", "/tmp/");
            builder.AddEnv("CHEETAH_MATCHES_REGISTRY_INTERNAL_SERVICE_HOST", "cheetah-matches-stub-registry");
            builder.AddEnv("CHEETAH_MATCHES_REGISTRY_INTERNAL_SERVICE_PORT", InternalGrpcPort.ToString());
            builder.AddVolumeMappings(roomsConfigPath, "/tmp/");
        }
    }
}