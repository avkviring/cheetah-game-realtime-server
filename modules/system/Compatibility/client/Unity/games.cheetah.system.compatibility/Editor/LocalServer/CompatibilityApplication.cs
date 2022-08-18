using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;
using UnityEngine;

namespace Cheetah.System.Compatibility.Editor.LocalServer
{
    public class CompatibilityApplication : PlatformApplication
    {
        private readonly string configurationPath = PlatformConfiguration.MakeHostedDataPath("system-compatibility/");

        public CompatibilityApplication() : base("system-compatibility")
        {
            ExternalGrpcServices.Add("cheetah.system.compatibility");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            Debug.Log("Configure CompatibilityApplication");
            base.ConfigureDockerContainerBuilder(builder);
            builder.AddVolumeMappings(configurationPath, "/tmp/");
            builder.AddEnv("CONFIG_FILE", "/tmp/versions.yaml");
        }
    }
}