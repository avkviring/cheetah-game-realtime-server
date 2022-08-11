using System;
using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;
using JetBrains.Annotations;

namespace Cheetah.Platform.Editor.LocalServer
{
    /// <summary>
    /// Серверное приложение
    /// 
    /// - конфигурация для запуска приложения в Docker
    ///
    /// - методы для работы с приложением
    ///  
    /// </summary>
    public abstract class ServerApplication
    {
        /// <summary>
        /// Уникальное имя приложения
        /// </summary>
        public string Name { get; }

        /// <summary>
        /// Docker образ приложения
        /// </summary>
        public virtual DockerImage DockerImage { get; }

        [CanBeNull] public string PostgresDatabase { get; internal set; }

        protected ServerApplication(string name)
        {
            Name = name;
        }


        public void EnablePostgreSQL(string database)
        {
            PostgresDatabase = database;
        }


        /// <summary>
        /// Вызывается после регистрации всех приложений
        /// 
        /// - может вызываться несколько раз
        /// 
        /// </summary>
        /// <param name="applications"></param>
        public virtual void ConfigureFromApplications(IList<ServerApplication> applications)
        {
        }

        /// <summary>
        /// Конфигурация для запуска Docker контейнера
        /// </summary>
        /// <returns></returns>
        public virtual void ConfigureDockerContainerBuilder(DockerContainerBuilder builder)
        {
        }

        public void ConfigurePostgresEnv(DockerContainerBuilder builder)
        {
            builder.AddEnv("POSTGRES_HOST", PostgreSqlApplication.Host);
            builder.AddEnv("POSTGRES_PORT", "5432");
            builder.AddEnv("POSTGRES_DB", PostgresDatabase);
            builder.AddEnv("POSTGRES_USER", PostgreSqlApplication.User);
            builder.AddEnv("POSTGRES_PASSWORD", PostgreSqlApplication.Password);
        }


        /// <summary>
        /// Преобразование лога из Docker контейнера в лог для Unity
        /// - определения канала для лога (Error, Info, ...)
        /// - фильтрация лога 
        /// </summary>
        /// <param name="log"></param>
        /// <returns>null - лог не будет отображен в консоле Unity</returns>
        public abstract LogItem? ConvertToLogItem(string log);


        /// <summary>
        /// Получить сообщение об ошибке для отображения в UI
        /// </summary>
        /// <param name="e"></param>
        /// <returns>null - не отображать сообщение</returns>
        [CanBeNull]
        public virtual string GetCreateContainerErrorMessage(Exception e)
        {
            return null;
        }


        /// <summary>
        /// Список зависимостей приложения
        /// </summary>
        public readonly ISet<string> Dependencies = new HashSet<string>();

        /// <summary>
        /// Список GRPC сервисов приложения
        /// </summary>
        public readonly ISet<string> ExternalGrpcServices = new HashSet<string>();

        public readonly ISet<string> AdminGrpcServices = new HashSet<string>();

        public struct LogItem
        {
            public string Log;
            public LogItemType ItemType;
        }

        public enum LogItemType
        {
            /// <summary>
            /// INFO канал, если не включено отображение INFO логов, то не будет отображен
            /// </summary>
            Info,

            /// <summary>
            /// ERROR канал, отображается всегда
            /// </summary>
            Error,

            /// <summary>
            /// INFO канал, отображается всегда
            /// </summary>
            Message
        }
    }
}