using System.IO;
using System.Text;
using Cheetah.Platform.Editor.LocalServer.Docker;
using JetBrains.Annotations;
using UnityEngine;

namespace Tests.Helpers
{
    /// <summary>
    /// Конфигурация интеграционного теста
    /// </summary>
    public class IntegrationTestConfigurator
    {
        /// <summary>
        /// Адрес внешнего сервера для тестирования
        /// Если null - то запускается локальный сервер
        /// </summary>
        [CanBeNull] public string ServerHost;

        /// <summary>
        /// Версия образов сервисов локального сервера
        /// </summary>
        [CanBeNull] public string ServerImageVersion;

        [CanBeNull] public string DockerMountDir;

        [CanBeNull]
        public static IntegrationTestConfigurator Load()
        {
            var fileName = Path.GetFullPath(Path.Combine(Application.dataPath, "../integration-test-config.json"));
            if (!File.Exists(fileName)) return null;
            var json = Encoding.Default.GetString(File.ReadAllBytes(fileName));
            var integrationTestConfigurator = JsonUtility.FromJson<IntegrationTestConfigurator>(json);
            DockerContainerBuilder.DockerMountDir = integrationTestConfigurator.DockerMountDir;
            return integrationTestConfigurator;
        }
    }
}