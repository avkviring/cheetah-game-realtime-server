using Cheetah.Platform.Editor.LocalServer.Docker;
using Cheetah.Platform.Editor.LocalServer.Runner;
using Cheetah.Platform.Editor.UIElements;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.GrpcProxy
{
    /// <summary>
    /// UI для настройки общих параметров микросервисов
    /// - хост/порт для grpc/http прокси
    /// - включение INFO логов для сервисов
    /// </summary>
    public class GrpcProxyApplicationConfigurator : IApplicationsConfigurator, GrpcProxyApplication.IConfig
    {
        private const string ShowInfoLogsPrefsKey = "cheetah.platform.microservice.show_info_logs";
        public string Title => "GRPC Services";
        public string Host => address?.Host ?? "127.0.0.1";
        public int Port => address?.Port ?? 7777;

        private NetworkAddressElement address;


        public VisualElement CreateUI()
        {
            VisualTreeAsset uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/LocalServer/GrpcProxy/UI.uxml");
            var ui = uiAsset.Instantiate();

            var showInfoLogs = ui.Q<Toggle>("show_info_logs");
            var showInfoLogValue = EditorPrefs.GetBool(ShowInfoLogsPrefsKey, false);
            DockerLogWatcher.ShowInfoLogs = showInfoLogValue;
            showInfoLogs.value = showInfoLogValue;
            showInfoLogs.RegisterCallback<ChangeEvent<bool>>(e =>
            {
                EditorPrefs.SetBool(ShowInfoLogsPrefsKey, e.newValue);
                DockerLogWatcher.ShowInfoLogs = e.newValue;
            });

            address = ui.Q<NetworkAddressElement>("address");

            return ui;
        }

        public void OnUpdateStatus(Status status)
        {
            address.SetEnabled(status != Status.Starting && status != Status.Started);
        }

        public int Order { get; } = int.MaxValue;
    }
}