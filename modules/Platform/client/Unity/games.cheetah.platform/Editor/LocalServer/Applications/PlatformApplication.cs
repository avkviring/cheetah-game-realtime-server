using Cheetah.Platform.Editor.Configuration;
using Cheetah.Platform.Editor.LocalServer.Docker;
using JetBrains.Annotations;
using UnityEditor.PackageManager;

namespace Cheetah.Platform.Editor.LocalServer.Applications
{
    /// <summary>
    /// Унифицированное приложение платформы
    /// </summary>
    public abstract class PlatformApplication : ServerApplication
    {
        protected const ushort InternalGrpcPort = 5001;

        [CanBeNull] public static string StaticPlatformImageVersion;

        protected override string DockerImageVersion => StaticPlatformImageVersion ?? base.DockerImageVersion;


        protected PlatformApplication(string containerName) : base(containerName, type =>
            $"ghcr.io/cheetah-game-platform/platform/{containerName}:{(StaticPlatformImageVersion ?? PackageInfo.FindForAssembly(type.Assembly).version)}")
        {
            var unityPackageId = PackageInfo.FindForAssembly(GetType().Assembly).assetPath;
            ConfigurationUtils.InitConfigDirectoryIfNotExists(unityPackageId, containerName);
        }


        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.AddCommand("/cheetah-" + ContainerName + "-server");
        }
    }
}