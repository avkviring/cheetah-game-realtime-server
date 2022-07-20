using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.System.Compatibility.Editor.LocalServer
{
    public class CompatibilityApplication : PlatformApplication
    {
        private readonly string configurationPath = ConfigurationPaths.MakeHostedDataPath("system-compatibility/");

        public CompatibilityApplication() : base("cheetah-system-compatibility")
        {
            ExternalGrpcServices.Add("cheetah.system.compatibility");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddVolumeMappings(configurationPath, "/tmp/");
            builder.AddEnv("CONFIG_FILE", "/tmp/versions.yaml");
        }
    }
}