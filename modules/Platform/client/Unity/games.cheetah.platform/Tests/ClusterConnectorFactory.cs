namespace Cheetah.Platform.Tests
{
    /// <summary>
    /// Создание соединение к кластеру с поддержкой интеграционных тестов из CI
    /// </summary>
    public static class ClusterConnectorFactory
    {
        public static ClusterConnector FromConfigFile()
        {
            var config = ClusterHostConfig.Load();
            return new ClusterConnector(config.ClusterHost, 443, true);
        }
    }
}