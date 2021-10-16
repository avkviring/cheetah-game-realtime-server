using System.Threading.Tasks;
using Cheetah.Platform;
using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.CheetahRegistry;
using Cheetah.Platform.Editor.LocalServer.Runner;
using UnityEngine;

namespace Tests.Helpers
{
    /// <summary>
    /// Создание соединение к кластеру с поддержкой интеграционных тестов из CI
    /// - если в интеграционных тестах не задан адрес сервера - то запускаем локальный сервер
    /// - обязательно вызывать connector.Shutdown() в конце теста, иначе Unity не сможет выйти после теста
    /// </summary>
    public class ConnectorFactory : IDockerProgressListener
    {
        public GRPCConnector GrpcConnector { get; private set; }

        public void SetProgressTitle(string title)
        {
            Debug.Log("Docker progress set title " + title);
        }

        public void SetProgress(int percent)
        {
            Debug.Log("Docker progress percent " + percent);
        }

        public async Task Connect()
        {
            var testConfiguration = IntegrationTestConfigurator.Load();
            if (testConfiguration == null)
            {
                PlatformApplication.ImageVersion = null;
                GrpcConnector = CreateLocalConnector();
            }
            else
            {
                if (testConfiguration.ServerHost != null)
                {
                    GrpcConnector = new GRPCConnector(testConfiguration.ServerHost, 443, true);
                }
                else
                {
                    PlatformApplication.ImageVersion = testConfiguration.ServerImageVersion;
                    var dockerRunner = new DockerServerRunner(CheetahRegistrySettingsFromConfig.Load());
                    await dockerRunner.DeterminationState();
                    if (dockerRunner.Status != Status.Started)
                    {
                        await dockerRunner.Restart(this);
                    }
                    GrpcConnector = CreateLocalConnector();
                }
            }
        }

        private static GRPCConnector CreateLocalConnector()
        {
            return new GRPCConnector("127.0.0.1", 7777, false);
        }
    }
}