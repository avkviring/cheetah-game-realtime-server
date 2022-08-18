using Cheetah.Platform.Editor.Configuration;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Factory.Editor.LocalServer.Factory
{
    public class FactoryApplication : PlatformApplication
    {
        public const string ServerName = "matches-factory";
        private const string PackageId = "games.cheetah.matches.factory";
        private readonly string roomsConfigPath = ConfigurationUtils.GetPathToConfigDirectory(ServerName);

        public FactoryApplication() : base(ServerName)
        {
            Dependencies.Add("matches-stubregistry");
            AdminGrpcServices.Add("cheetah.matches.factory.admin");
            ConfigurationUtils.InitConfigDirectoryIfNotExists(PackageId, ServerName);
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddEnv("TEMPLATES_PATH", "/tmp/rooms/");
            builder.AddEnv("CHEETAH_MATCHES_REGISTRY_INTERNAL_SERVICE_HOST", "matches-stubregistry");
            builder.AddEnv("CHEETAH_MATCHES_REGISTRY_INTERNAL_SERVICE_PORT", InternalGrpcPort.ToString());
            builder.AddVolumeMappings(roomsConfigPath, "/tmp/");
        }
    }
}