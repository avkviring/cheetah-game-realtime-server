using Cheetah.Matches.Realtime.Editor.LocalServer.Application;
using Cheetah.Platform.Editor.LocalServer;
using Cheetah.Platform.Editor.LocalServer.Runner;
using Cheetah.Platform.Editor.UIElements;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.LocalServer
{
    public class RelayApplicationsConfigurator : IApplicationsConfigurator, RelayApplication.IConfig
    {
        private NetworkAddressElement address;


        public string Host => address != null ? address.Host : "127.0.0.1";
        public int Port => address?.Port ?? 7777;
        public string Title { get; } = "Matches Relay";


        public VisualElement CreateUI()
        {
            var ui = AssetDatabase
                .LoadAssetAtPath<VisualTreeAsset>("Packages/games.Cheetah.Matches.Realtime/Editor/LocalServer/UI.uxml")
                .Instantiate();
            address = ui.Q<NetworkAddressElement>("address");

            return ui;
        }

        public void OnUpdateStatus(Status status)
        {
            var stopped = status != Status.Starting && status != Status.Started;
            address.SetEnabled(stopped);
        }

        public int Order { get; } = 0;
    }
}