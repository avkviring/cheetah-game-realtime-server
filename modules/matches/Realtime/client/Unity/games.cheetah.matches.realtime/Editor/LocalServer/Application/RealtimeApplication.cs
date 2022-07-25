using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Realtime.Editor.LocalServer.Application
{
    public class RealtimeApplication : PlatformApplication
    {

        public interface IConfig
        {
            public string Host { get; }
            public int Port { get; }
        }

        public readonly IConfig Config;

        public const string AppName = "matches-realtime";

        public RealtimeApplication(IConfig config) : base(AppName)
        {
            Config = config;
            AdminGrpcServices.Add("cheetah.matches.realtime.admin");
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            base.ConfigureDockerContainerBuilder(builder);

            var host = Config.Host;
            var port = Config.Port;
            builder.AddPortMapping(Protocol.Udp, 5555, host, port);
        }


        public override LogItem? ConvertToLogItem(string log)
        {
            if (log.Contains("[room") && log.Contains("INFO"))
            {
                return new LogItem
                {
                    Log = log.Replace("INFO - ", ""),
                    ItemType = LogItemType.Message
                };
            }

            return base.ConvertToLogItem(log);
        }
    }
}