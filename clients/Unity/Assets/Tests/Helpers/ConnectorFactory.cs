using System.Threading.Tasks;
using Cheetah.Platform;
#if UNITY_EDITOR
using Cheetah.Platform.Editor.LocalServer.CheetahRegistry;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Runner;
#endif

namespace Tests.Helpers
{
    /// <summary>
    /// Создание соединение к кластеру с поддержкой интеграционных тестов из CI
    /// - если в интеграционных тестах не задан адрес сервера - то запускаем локальный сервер
    /// - обязательно вызывать connector.Shutdown() в конце теста, иначе Unity не сможет выйти после теста
    /// </summary>
    public class ConnectorFactory
    {
        public ClusterConnector ClusterConnector { get; private set; }

        public void SetProgressTitle(string title)
        {
        }

        public void SetProgress(int percent)
        {
        }

        public async Task Connect()
        {
            var testConfiguration = IntegrationTestConfigurator.Load();
            if (testConfiguration == null)
            {
#if UNITY_EDITOR
                PlatformApplication.ImageVersion = null;
                ClusterConnector = CreateLocalConnector();
#endif
            }
            else
            {
                if (testConfiguration.ServerHost != null)
                {
                    ClusterConnector = new ClusterConnector(testConfiguration.ServerHost, 443, true);
                }
                else
                {
#if UNITY_EDITOR
                    PlatformApplication.ImageVersion = testConfiguration.ServerImageVersion;
                    var dockerRunner = new DockerServerRunner(CheetahRegistrySettingsFromConfig.Load());
                    await dockerRunner.DeterminationState();
                    if (dockerRunner.Status != Status.Started)
                    {
                        await dockerRunner.Restart(new StubDockerProgressListener());
                    }

                    ClusterConnector = CreateLocalConnector();
#endif
                }
            }
        }

        private static ClusterConnector CreateLocalConnector()
        {
            return new ClusterConnector("127.0.0.1", 7777, false);
        }
    }
}

#if UNITY_EDITOR
class StubDockerProgressListener : IDockerProgressListener
{
    public void SetProgressTitle(string title)
    {
    }

    public void SetProgress(int percent)
    {
    }
}
#endif