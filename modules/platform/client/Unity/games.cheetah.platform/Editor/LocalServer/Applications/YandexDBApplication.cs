using System.Collections.Generic;
using System.Text;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Platform.Editor.LocalServer.Applications
{
    /// <summary>
    ///     Приложение для запуска yandex db
    /// </summary>
    public class YandexDBApplication : ServerApplication
    {
        

        public YandexDBApplication() : base("ydb")
        {

        }

        public override DockerImage DockerImage => DockerImage.From("cr.yandex/yc/yandex-docker-local-ydb:latest");


        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.AddEnv("YDB_USE_IN_MEMORY_PDISKS", "true");
        }


        public override LogItem? ConvertToLogItem(string log)
        {
            return new LogItem
            {
                Log = log,
                ItemType = GetLogItemType(log)
            };
        }

        private LogItemType GetLogItemType(string log)
        {
            return log.Contains("FATAL") || log.Contains("ERROR") ? LogItemType.Error : LogItemType.Info;
        }
    }
}