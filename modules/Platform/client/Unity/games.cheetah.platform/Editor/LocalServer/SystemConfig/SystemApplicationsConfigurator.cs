using Cheetah.Platform.Editor.LocalServer.Applications;
using Cheetah.Platform.Editor.LocalServer.Docker;
using Cheetah.Platform.Editor.UIElements;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.SharedConfig
{
    /// <summary>
    /// UI для настройки общих параметров микросервисов
    /// - хост/порт для grpc/http прокси
    /// - включение INFO логов для сервисов
    /// </summary>
    public class SystemApplicationsConfigurator : IApplicationsConfigurator, GrpcProxyApplication.IConfig
    {
        private const string ShowInfoLogsPrefsKey = "cheetah.platform.microservice.show_info_logs";
        public string Title => "System settings";
        public string Host => address?.Host ?? "127.0.0.1";
        public int Port => address?.Port ?? 7777;

        public bool KeepFailedContainers => keepFailedContainersUI.value;
        public bool ShowInfoLogs => showInfoLogsUI.value;

        private NetworkAddressElement address;
        private Toggle keepFailedContainersUI;
        private Toggle showInfoLogsUI;


        public VisualElement CreateUI()
        {
            VisualTreeAsset uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/LocalServer/SystemConfig/UI.uxml");
            var ui = uiAsset.Instantiate();
            showInfoLogsUI = ui.Q<Toggle>("show_info_logs");
            var showInfoLogValue = EditorPrefs.GetBool(ShowInfoLogsPrefsKey, false);
            showInfoLogsUI.value = showInfoLogValue;
            showInfoLogsUI.RegisterCallback<ChangeEvent<bool>>(e => { EditorPrefs.SetBool(ShowInfoLogsPrefsKey, e.newValue); });
            keepFailedContainersUI = ui.Q<Toggle>("keep_failed_containers");
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