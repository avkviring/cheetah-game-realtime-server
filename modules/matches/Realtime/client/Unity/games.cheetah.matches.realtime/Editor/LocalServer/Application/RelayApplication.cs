using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Matches.Relay.Editor.LocalServer.Application
{
    public class RelayApplication : PlatformApplication
    {

        public interface IConfig
        {
            public string Host { get; }
            public int Port { get; }
        }

        public readonly IConfig Config;

        public const string AppName = "cheetah-matches-relay";

        public RelayApplication(IConfig config) : base("cheetah-matches-relay")
        {
            Config = config;
            AdminGrpcServices.Add("cheetah.matches.relay.admin");
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