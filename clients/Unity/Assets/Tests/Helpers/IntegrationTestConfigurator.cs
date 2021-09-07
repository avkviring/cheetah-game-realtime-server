using System.IO;
using System.Text;
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

        [CanBeNull]
        public static IntegrationTestConfigurator Load()
        {
            var fileName = Path.GetFullPath(Path.Combine(Application.dataPath, "../integration-test-config.json"));
            Debug.Log(">>> IntegrationTestConfigurator filename " + fileName);
            if (File.Exists(fileName))
            {
                var json = Encoding.Default.GetString(File.ReadAllBytes(fileName));
                Debug.Log(">>> json " + json);
                return JsonUtility.FromJson<IntegrationTestConfigurator>(json);
            }
            else
            {
                return null;
            }
        }
    }
}