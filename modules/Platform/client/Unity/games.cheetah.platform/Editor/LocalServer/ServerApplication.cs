using System;
using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer.Docker;
using JetBrains.Annotations;
using UnityEditor.PackageManager;

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
        private readonly string dockerImageReference;

        /// <summary>
        /// Уникальное имя приложения
        /// </summary>
        public string ContainerName { get; }

        protected virtual string DockerImageVersion => PackageInfo.FindForAssembly(GetType().Assembly).version;

        /// <summary>
        /// Docker образ приложения
        /// </summary>
        public DockerImage DockerImage => DockerImage.From(dockerImageReference + ":" + DockerImageVersion);


        protected ServerApplication(string containerName, string dockerImageReference)
        {
            this.dockerImageReference = dockerImageReference;
            ContainerName = containerName;
        }

        protected ServerApplication(string containerName, Func<Type, string> dockerImageReferenceBuilder)
        {
            dockerImageReference = dockerImageReferenceBuilder.Invoke(GetType());
            ContainerName = containerName;
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


        /// <summary>
        /// Преобразование лога из Docker контейнера в лог для Unity
        /// - определения канала для лога (Error, Info, ...)
        /// - фильтрация лога 
        /// </summary>
        /// <param name="log"></param>
        /// <returns>null - лог не будет отображен в консоле Unity</returns>
        public virtual LogItem? ConvertToLogItem(string log)
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