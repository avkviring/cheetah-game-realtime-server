using System;
using System.IO;
using System.Text;
using UnityEngine;

namespace Cheetah.Platform.Tests
{
    /// <summary>
    /// Конфигурация интеграционного теста
    /// </summary>
    public class ClusterHostConfig
    {
        /// <summary>
        /// Адрес внешнего сервера для тестирования
        /// </summary>
        public string ClusterHost;


        public static ClusterHostConfig Load()
        {
            var fileName = Path.GetFullPath(Path.Combine(Application.dataPath, "../integration-test-config.json"));
            if (!File.Exists(fileName))
                throw new Exception(fileName + " not found.");
            var json = Encoding.Default.GetString(File.ReadAllBytes(fileName));
            var integrationTestConfigurator = JsonUtility.FromJson<ClusterHostConfig>(json);
            return integrationTestConfigurator;
        }
    }
}
