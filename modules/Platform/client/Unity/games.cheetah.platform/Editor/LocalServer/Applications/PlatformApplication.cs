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

        protected PlatformApplication(string name) : base(name) { }

        [CanBeNull] public static string ImageVersion;

        public override DockerImage DockerImage => DockerImage.From(
            "ghcr.io/cheetah-game-platform/platform",
            Name,
            ImageVersion ?? PackageInfo.FindForAssembly(GetType().Assembly).version);


        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.AddCommand("/cheetah-" + Name + "-server");
        }

        public override LogItem? ConvertToLogItem(string log)
        {
            var upperLog = log.ToUpper();
            return new LogItem
            {
                Log = log.Replace("INFO - ", "").Replace("ERROR - ", ""),
                ItemType = upperLog.Contains("FATAL") || upperLog.Contains("ERROR") || upperLog.Contains("PANICKED")
                    ? LogItemType.Error
                    : LogItemType.Info
            };
        }
    }
}