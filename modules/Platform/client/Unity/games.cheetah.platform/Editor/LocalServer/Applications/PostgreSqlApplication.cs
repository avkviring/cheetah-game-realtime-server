using System.Collections.Generic;
using System.Text;
using Cheetah.Platform.Editor.LocalServer.Docker;

namespace Cheetah.Platform.Editor.LocalServer.Applications
{
    /// <summary>
    ///     Приложение для запуска https://www.postgresql.org
    /// </summary>
    public class PostgreSqlApplication : ServerApplication
    {
        public const string Host = "postgres";
        public const string User = "user";
        public const string Password = "password";
        private readonly List<string> databases;

        public PostgreSqlApplication(List<string> databases) : base(Host, "postgres:13.1-alpine")
        {
            this.databases = databases;
        }


        public override void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
            builder.AddEnv("POSTGRES_USER", User);
            builder.AddEnv("POSTGRES_PASSWORD", Password);
            builder.SetHealthCheck(new List<string>
            {
                "CMD-SHELL", "pg_isready -U " + User
            });

            var script = new StringBuilder();
            foreach (var database in databases)
                script.AppendLine("CREATE DATABASE " + database + ";");

            builder.AddVolumeContentMappings(script.ToString(), "/docker-entrypoint-initdb.d/create.sql");
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
            if (log.Contains("ON_ERROR_STOP")) return LogItemType.Info;
            // нормальная ситуация при первом запуске
            if (log.Contains("does not exist")) return LogItemType.Info;

            // перезагрузка postgresql в процессе создания первой базы
            if (log.Contains("the database system is starting up")) return LogItemType.Info;

            // перезагрузка postgresql в процессе создания первой базы
            if (log.Contains("the database system is shutting down")) return LogItemType.Info;


            if (log.Contains("FATAL") || log.Contains("ERROR")) return LogItemType.Error;


            return LogItemType.Info;
        }
    }
}