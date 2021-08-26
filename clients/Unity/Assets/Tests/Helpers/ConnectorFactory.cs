using System.IO;
using System.Text;
using Cheetah.Platform;
using UnityEditor;
using UnityEngine;

namespace Tests.Helpers
{
    /// <summary>
    /// Создает connector к кластеру для тестов
    /// - если есть файл "/tmp/cheetah-unity-test-settings.json" - то адрес кластера читается из него
    /// - иначе используется параметры для подключения к локальному кластеру
    /// - файл с параметрами подключения в основном используется в CI для подключенния к внешнему кластеру
    /// </summary>
    public static class ConnectorFactory
    {
        private class ConnectorConfiguration
        {
            public string ServerHost;
            public int ServerPort;
            public bool UseSSL;
        }

        public static Connector Create()
        {
            var fileName = Path.GetFullPath(Application.dataPath+"/../../../cheetah-unity-test-settings.json");
            var testConfiguration = File.Exists(fileName)
                ? JsonUtility.FromJson<ConnectorConfiguration>(Encoding.Default.GetString(File.ReadAllBytes(fileName)))
                : new ConnectorConfiguration
                {
                    ServerHost = "localhost",
                    ServerPort = 7777,
                    UseSSL = false
                };
            return new Connector(testConfiguration.ServerHost, testConfiguration.ServerPort, testConfiguration.UseSSL);
        }
    }
}