using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cheetah.Matches.Factory.Editor.Configurations;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Provider;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Users;
using Cheetah.Matches.Realtime.Editor.GRPC;
using Cheetah.Matches.Realtime.Editor.UIElements;
using Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector;
using Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector.Provider;
using Cheetah.Matches.Realtime.Editor.UIElements.StatusIndicator;
using Cheetah.Platform.Editor.Connector;
using Grpc.Core;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer
{
    /// <summary>
    /// Панель просмотра сетевых команд с сервера
    /// </summary>
    public class DumpViewerWindow : EditorWindow
    {
        /// <summary>
        /// Удаленные провайдеры данных или тестовые
        /// </summary>
        private const bool RemoteProviders = true;

        private const string PathToUxml = "Packages/games.Cheetah.Matches.Realtime/Editor/DumpViewer/UI/Window.uxml";
        private const int UpdateTime = 60 / 2;
        private const int ErrorUpdateTime = 60;

        [MenuItem("Window/Cheetah/Relay dump viewer")]
        public static void ShowWindow()
        {
            var window = GetWindow<DumpViewerWindow>();
            window.titleContent = new GUIContent("Relay dump viewer");
        }

        private DumpProvider provider;
        private ObjectsViewer objectsViewer;
        private UsersViewer usersViewer;
        private StatusIndicator statusIndicator;
        private RoomsSelector roomSelector;
        private int timeToUpdate;
        private ToolbarButton dumpButton;
        private ToolbarButton exportButton;
        private ToolbarToggle objectsTab;
        private ToolbarToggle usersTab;
        private Toggle autoRefresh;
        private VisualElement tabContent;
        private ulong? selectedRoom;
        private bool connectError;
        private RoomsProvider roomsProvider;
        private ConfigurationsProvider configurationsProvider;
        private DumpResponse dump;

        public void OnEnable()
        {
            var ui = LoadVisualTree();
            rootVisualElement.Add(ui);
            statusIndicator = ui.Q<StatusIndicator>("status");
            provider = RemoteProviders
                ? (DumpProvider)new RemoteDumpProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestDumpProvider();
            configurationsProvider = RemoteProviders
                ? (ConfigurationsProvider)new RemoteConfigurationsProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestConfigurationsProvider();
            ConfigureRoomSelector(ui);
            ConfigureDumpButton(ui);
            ConfigureTabs(ui);
            ConfigureExportButton(ui);
            ConfigureHelpButton(ui);
        }

        private void ConfigureHelpButton(VisualElement ui)
        {
            ui.Q<ToolbarButton>("help").RegisterCallback<ClickEvent>(evt =>
            {
                Application.OpenURL("https://docs.cheetah.games/components/relay/develop/dump-panel/");
            });
        }

        private void ConfigureExportButton(VisualElement ui)
        {
            ui.Q<ToolbarButton>("export").RegisterCallback<ClickEvent>(evt =>
            {
                if (dump != null)
                {
                    var path = EditorUtility.SaveFolderPanel(
                        "Select folders for store dumps (objects.csv, users.csv)",
                        "",
                        "");
                    if (path.Length == 0) return;
                    

                    var objects = new StringBuilder();
                    objects.Append("id").Append("\t");
                    objects.Append("template").Append("\t");
                    objects.Append("ownerUserId").Append("\t");
                    objects.Append("groups").Append("\t");
                    objects.Append("created").Append("\t");
                    objects.Append("fields").Append("\t");
                    objects.Append("compareAndSetOwners");
                    objects.AppendLine();
                    foreach (var dumpObject in dump.Objects)
                    {
                        objects.Append(dumpObject.Id).Append("\t");
                        objects.Append(dumpObject.Template).Append("\t");
                        objects.Append(dumpObject.HasOwnerUserId ? dumpObject.OwnerUserId : "None").Append("\t");
                        objects.Append(dumpObject.Groups).Append("\t");
                        objects.Append(dumpObject.Created).Append("\t");
                        objects.Append(string.Join(",", dumpObject.Fields.Select(p => p.Id + "=" + p.Value))).Append("\t");
                        objects.Append(string.Join(",", dumpObject.CompareAndSetOwners.Select(p => p.Key + "=" + p.Value)));
                        objects.AppendLine();
                    }
                    
                    var users = new StringBuilder();
                    users.Append("id").Append("\t");
                    users.Append("groups").Append("\t");
                    users.Append("attached").Append("\t");
                    users.Append("compareAndSetCleaners");
                    users.AppendLine();
                    foreach (var dumpUser in dump.Users)
                    {
                        users.Append(dumpUser.Id).Append("\t");
                        users.Append(dumpUser.Groups).Append("\t");
                        users.Append(dumpUser.Attached).Append("\t");
                        users.Append(string.Join(",",
                            dumpUser.CompareAndSetCleaners.Select(p => (p.GameObjectId, p.GameObjectOwnerUser, p.FieldId, p.Value)))).Append("\t");
                        users.AppendLine();
                    }
                    
                    File.WriteAllText(path+"/objects.csv", objects.ToString());
                    File.WriteAllText(path+"/users.csv", users.ToString());
                }
            });
        }

        private void ConfigureTabs(VisualElement ui)
        {
            objectsViewer = new ObjectsViewer(statusIndicator, configurationsProvider);
            usersViewer = new UsersViewer(configurationsProvider);
            tabContent = ui.Q<VisualElement>("tabs-content");
            autoRefresh = ui.Q<Toggle>("auto-refresh");
            var tabsController = new TabsController();
            objectsTab = ui.Q<ToolbarToggle>("objectsTab");
            tabsController.RegisterTab(objectsTab, () => { SwitchContentTo(tabContent, objectsViewer); });
            usersTab = ui.Q<ToolbarToggle>("usersTab");
            tabsController.RegisterTab(usersTab, () => { SwitchContentTo(tabContent, usersViewer); });
            tabsController.SwitchToFirst();
            DisableContent();
        }

        private void ConfigureDumpButton(VisualElement ui)
        {
            dumpButton = ui.Q<ToolbarButton>("dump-button");
            dumpButton.SetEnabled(false);
            dumpButton.RegisterCallback<ClickEvent>(OnDumpButtonPressed);
        }

        private void ConfigureRoomSelector(VisualElement ui)
        {
            roomsProvider = RemoteProviders
                ? (RoomsProvider)new RemoteRoomsProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestRoomsProvider();
            roomSelector = ui.Q<RoomsSelector>("room-selector");
            roomSelector.SetProvider(roomsProvider);
            roomSelector.RoomSelectEvent += RoomSelect;
            roomSelector.RoomUnselectEvent += RoomUnselect;
        }

        private void RoomUnselect()
        {
            selectedRoom = null;
            dumpButton.SetEnabled(false);
        }

        private Task RoomSelect(ulong room)
        {
            selectedRoom = room;
            dumpButton.SetEnabled(!connectError);
            return configurationsProvider.Load();
        }

        private async void OnDumpButtonPressed(ClickEvent evt)
        {
            DisableContent();
            try
            {
                var first = true;
                while (autoRefresh.value || first)
                {
                    statusIndicator.ResetStatus();
                    var dump = await provider.Dump((ulong)selectedRoom);
                    objectsViewer.SetData(dump);
                    usersViewer.SetData(dump);
                    this.dump = dump;
                    await Task.Delay(60);
                    first = false;
                }

                EnableContent();
            }
            catch (RpcException e)
            {
                var emptyDump = new DumpResponse();
                objectsViewer.SetData(emptyDump);
                usersViewer.SetData(emptyDump);
                Debug.LogError(e);
                statusIndicator.SetStatus("Get dump error  " + e.Status.Detail, StatusIndicator.MessageType.Error);
            }
        }

        private void EnableContent()
        {
            dumpButton.SetEnabled(true);
        }

        private void DisableContent()
        {
            dumpButton.SetEnabled(false);
        }


        private static void SwitchContentTo(VisualElement content, VisualElement element)
        {
            content.Clear();
            content.Add(element);
        }


        private static TemplateContainer LoadVisualTree()
        {
            var uiAsset = AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(PathToUxml);
            return uiAsset.CloneTree();
        }


        private async void Update()
        {
            timeToUpdate++;
            if (timeToUpdate < UpdateTime)
            {
                return;
            }

            if (connectError && timeToUpdate < ErrorUpdateTime)
            {
                return;
            }

            timeToUpdate = 0;
            try
            {
                await roomSelector.Update();
                ConnectedState();
            }
            catch (RpcException e)
            {
                Debug.Log(e);
                ConnectErrorState();
            }
        }

        private void ConnectedState()
        {
            if (!connectError)
            {
                return;
            }

            connectError = false;
            roomSelector.SetEnabled(true);
            dumpButton.SetEnabled(selectedRoom != null);
        }

        private void ConnectErrorState()
        {
            connectError = true;
            roomSelector.SetEnabled(false);
            dumpButton.SetEnabled(false);
            statusIndicator.SetStatus("Cannot connect to cluster.", StatusIndicator.MessageType.Error);
        }


        private async void OnDestroy()
        {
            await provider.Destroy();
            await configurationsProvider.Destroy();
        }
    }
}