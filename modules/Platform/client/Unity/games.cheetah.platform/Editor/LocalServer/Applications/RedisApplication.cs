using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Platform.Editor.LocalServer.Applications
{
    /// <summary>
    ///     Приложение для запуска https://redis.io
    /// </summary>
    public class RedisApplication : ServerApplication
    {
        public RedisApplication(string prefix) : base(prefix + "-redis")
        {
        }

        public override DockerImage DockerImage => DockerImage.From("redis:6.2.1");

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.SetHealthCheck(new List<string> { "CMD", "redis-cli", "ping" });
        }

        public override LogItem? ConvertToLogItem(string log)
        {
            return new LogItem
            {
                Log = log,
                ItemType = LogItemType.Info
            };
        }
    }
}