using System;
using System.Collections.Generic;
using System.Text;
using Cheetah.Platform.Editor.Connector;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Platform.Editor.LocalServer.Applications
{
    /// <summary>
    /// Контейнер для проксирования GPRC запросов к сервисам
    /// </summary>
    public class GrpcProxyApplication : ServerApplication
    {
        private readonly Dictionary<string, string> externalGrpcMappings = new();
        private readonly Dictionary<string, string> adminGrpcMappings = new();


        public interface IConfig
        {
            public string Host { get; }
            public int Port { get; }
        }

        private IConfig Config;

        public GrpcProxyApplication(IConfig config) : base("grpc_proxy")
        {
            Config = config;
        }

        public override DockerImage DockerImage => DockerImage.From("nginx:1.19.8");

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
            if (log.Contains("emerg")) return LogItemType.Error;
            if (log.Contains("error")) return LogItemType.Error;

            return LogItemType.Info;
        }

        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.AddPortMapping(Protocol.Tcp, 80, Config.Host, Config.Port);
            LocalClusterConnectorFactory.Configure(Config.Host, Config.Port);

            var mappings = new StringBuilder();
            foreach (var mapping in externalGrpcMappings)
            {
                AddMapping(mapping, mappings, 5000);
            }

            foreach (var mapping in adminGrpcMappings)
            {
                AddMapping(mapping, mappings, 5002);
            }

            string config = $@"
            events {{}}
            http {{
                log_format main $status $request;
                error_log /dev/stderr error;
                access_log /dev/stdout main;
                server {{
                    listen 80;
                    {mappings}
                }}
            }} ";

            builder.AddVolumeContentMappings(config, "/etc/nginx/nginx.conf");
        }

        private static void AddMapping(KeyValuePair<string, string> mapping, StringBuilder mappings, int port)
        {
            var path = mapping.Key;
            var service = mapping.Value;
            mappings.AppendLine($"location ^~/{path} {{ grpc_pass grpc://{service}:{port};}}");
        }

        public override string GetCreateContainerErrorMessage(Exception e)
        {
            return
                "Возможно адрес/порт заданы неправильно или уже используются.\n"
                + "Указать новый адрес/порт можно в разделе Services.";
        }

        public override void ConfigureFromApplications(IList<ServerApplication> applications)
        {
            Dependencies.Clear();

            externalGrpcMappings.Clear();
            adminGrpcMappings.Clear();

            foreach (var application in applications)
            {
                CollectGrpcServices(application, application.ExternalGrpcServices, externalGrpcMappings);
                CollectGrpcServices(application, application.AdminGrpcServices, adminGrpcMappings);
            }
        }

        private void CollectGrpcServices(
            ServerApplication application,
            ICollection<string> services,
            IDictionary<string, string> mapping
        )
        {
            foreach (var service in services)
            {
                mapping[service] = application.Name;
            }

            if (services.Count > 0)
            {
                Dependencies.Add(application.Name);
            }
        }
    }
}