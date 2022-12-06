using System;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using Cheetah.Platform;
using Games.Cheetah.Client.Editor.CommandsViewer.Provider;
using Games.Cheetah.Client.Editor.CommandsViewer.UI;
using Games.Cheetah.Client.Editor.CommandsViewer.UI.Controller;
using Games.Cheetah.Client.Editor.UIElements.HistoryTextField;
using Games.Cheetah.Client.Editor.UIElements.NetworkAddress;
using Games.Cheetah.Client.Editor.UIElements.RoomsSelector;
using Games.Cheetah.Client.Editor.UIElements.RoomsSelector.Provider;
using Games.Cheetah.Client.Editor.UIElements.StatusIndicator;
using Games.Cheetah.Client.Editor.UIElements.Table;
using Grpc.Core;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace Games.Cheetah.Client.Editor.CommandsViewer
{
    /// <summary>
    /// Панель просмотра сетевых команд с сервера
    /// </summary>
    public class NetworkCommandsViewerWindow : EditorWindow
    {
        /// <summary>
        /// Удаленные провайдеры данных или тестовые
        /// </summary>
        private const bool RemoteProviders = false;

        private const string PathToStyleSheet = "Packages/games.cheetah.client/Editor/CommandsViewer/UI/Styles.uss";
        private const string PathToUxml = "Packages/games.cheetah.client/Editor/CommandsViewer/UI/Window.uxml";
        private const int UpdateTime = 60 / 2;
        private const int ErrorUpdateTime = 60;

        private readonly Columns columns = new();
        private StatusIndicator statusIndicator;
        private RoomsSelector roomsSelector;
        private SearchFieldController searchFieldController;
        private ToolbarButton pauseButton;
        private NetworkAddress realtimeGrpcAdminNetworkAddress;
        private ToolbarButton clearButton;

        private TracedCommandsProvider provider;
        private TableController tableController;

        private int timeToUpdate;
        private bool pause;
        private bool connectError;
        private bool inUpdate;
        private TableElement table;
        private HistoryTextField searchTextField;


        [MenuItem("Window/Cheetah/Commands viewer")]
        public static void ShowServerCommandsViewerWindow()
        {
            var window = GetWindow<NetworkCommandsViewerWindow>();
            window.titleContent = new GUIContent("Cheetah CommandsViewer");
        }

        public void OnEnable()
        {
            inUpdate = false; // иначе он может остаться в true после перезагрузки домена
            LoadStyles();
            var ui = LoadVisualTree();
            new ColumnsMenuController(ui.Q<ToolbarMenu>("colummenu"), columns);
            statusIndicator = ui.Q<StatusIndicator>("status");
            realtimeGrpcAdminNetworkAddress = ui.Q<NetworkAddress>("server-address");
            roomsSelector = ui.Q<RoomsSelector>("room-selector");
            table = ui.Q<TableElement>("commands-table");
            searchTextField = ui.Q<HistoryTextField>("searchField");
            rootVisualElement.Add(ui);
            SetupConnect(ui);
            SetupClearButton(ui);
            SetupPauseButton(ui);
            SetupHelpButton(ui);
            SetupExportButton(ui);
        }

        private void SetupConnect(TemplateContainer ui)
        {
            var networkAddress = ui.Q<NetworkAddress>("server-address");
            networkAddress.AddConnectCallback(async address => await DoConnect(address));
        }


        private static void SetupHelpButton(VisualElement ui)
        {
            ui.Q<ToolbarButton>("help").RegisterCallback<ClickEvent>(evt =>
            {
                Application.OpenURL("https://docs.cheetah.games/components/relay/develop/commands-panel/");
            });
        }

        private void SetupPauseButton(VisualElement ui)
        {
            pauseButton = ui.Q<ToolbarButton>("pause");
            pauseButton.RegisterCallback<ClickEvent>(evt =>
            {
                pause = !pause;
                searchFieldController.Enabled(!pause);
            });
        }

        private void SetupClearButton(VisualElement ui)
        {
            clearButton = ui.Q<ToolbarButton>("clear");
            clearButton.RegisterCallback<ClickEvent>(evt =>
            {
                provider.GetCommands().Clear();
                tableController.Update();
            });
        }

        private void SetupExportButton(VisualElement ui)
        {
            ui.Q<ToolbarButton>("export").RegisterCallback<ClickEvent>(evt =>
            {
                var path = EditorUtility.SaveFilePanel(
                    "Export commands to file",
                    "",
                    "commands.csv",
                    "csv");
                if (path.Length == 0) return;
                var builder = new StringBuilder();
                builder.Append("time").Append("\t");
                builder.Append("user").Append("\t");
                builder.Append("direction").Append("\t");
                builder.Append("command").Append("\t");
                builder.Append("template").Append("\t");
                builder.Append("fieldId").Append("\t");
                builder.Append("fieldType").Append("\t");
                builder.Append("objectId").Append("\t");
                builder.Append("value");
                builder.AppendLine();
                foreach (var command in provider.GetCommands())
                {
                    builder.Append(command.Time).Append("\t");
                    builder.Append(command.UserId).Append("\t");
                    builder.Append(command.Direction).Append("\t");
                    builder.Append(command.Command_).Append("\t");
                    builder.Append(command.HasTemplate ? command.Template : "None").Append("\t");
                    builder.Append(command.HasFieldId ? command.FieldId : "None").Append("\t");
                    builder.Append(command.HasFieldType ? command.FieldType : "None").Append("\t");
                    builder.Append(command.ObjectId).Append("\t");
                    builder.Append(command.Value);
                    builder.AppendLine();
                }

                File.WriteAllText(path, builder.ToString());
            });
        }

        private void LoadStyles()
        {
            var uss = AssetDatabase.LoadAssetAtPath<StyleSheet>(PathToStyleSheet);
            rootVisualElement.styleSheets.Add(uss);
        }


        private static TemplateContainer LoadVisualTree()
        {
            var uiAsset = AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(PathToUxml);
            return uiAsset.CloneTree();
        }


        private async void Update()
        {
            if (provider == null)
            {
                return;
            }

            if (inUpdate)
            {
                return;
            }

            inUpdate = true;


            try
            {
                ConfigureEnabledStatus();

                if (pause)
                {
                    return;
                }

                timeToUpdate++;

                // если ошибка - то увеличиваем время обновления
                // для исключения спама ошибок
                if (connectError && (timeToUpdate <= ErrorUpdateTime))
                {
                    return;
                }

                if (timeToUpdate <= UpdateTime)
                {
                    return;
                }

                timeToUpdate = 0;

                try
                {
                    // обновляем данные
                    await roomsSelector.Update();
                    await searchFieldController.Update();

                    if (await provider.Update())
                    {
                        tableController.Update();
                    }

                    if (connectError)
                    {
                        statusIndicator.ResetStatus();
                    }

                    connectError = false;
                }
                catch (RpcException e)
                {
                    if (e.StatusCode == StatusCode.Unavailable)
                    {
                        Debug.Log(e);
                    }
                    else
                    {
                        Debug.LogError(e);
                    }

                    connectError = true;
                    statusIndicator.SetStatus("Cannot connect to cluster.", StatusIndicator.MessageType.Error);
                }
            }
            finally
            {
                inUpdate = false;
            }
        }

        private void ConfigureEnabledStatus()
        {
            if (searchFieldController != null)
            {
                searchFieldController.Enabled(provider != null && provider.IsReady() && !pause && !connectError);
            }

            pauseButton.SetEnabled(!connectError);

            var connected = !pause && !connectError;
            roomsSelector.SetEnabled(connected);
            clearButton.SetEnabled(connected);
            pauseButton.text = pause ? "Resume" : "Pause";
        }


        private async void OnDestroy()
        {
            if (provider != null)
            {
                await provider.Destroy();
            }
        }


        private async Task DoConnect(string address)
        {
            try
            {
                var uri = new Uri(address);
                var clusterConnector = new ClusterConnector(uri.Host, uri.Port, uri.Scheme == "https");
                var roomsProvider = RemoteProviders
                    ? (RoomsProvider)new RemoteRoomsProvider(clusterConnector)
                    : new TestRoomsProvider();

                await roomsProvider.GetRooms();

                roomsSelector.SetProvider(roomsProvider);
                provider = RemoteProviders
                    ? (TracedCommandsProvider)new RemoteTracedCommandsProvider(clusterConnector)
                    : new TestTracedCommandsProvider();

                roomsSelector.RoomSelectEvent += provider.SetRoom;
                roomsSelector.RoomUnselectEvent += provider.ResetRooms;


                tableController = new TableController(table, columns, provider);
                searchFieldController = new SearchFieldController(searchTextField, statusIndicator, provider);
                statusIndicator.SetStatus("Connected", StatusIndicator.MessageType.Regular);
            }
            catch (Exception e)
            {
                Debug.LogError(e);
                var message = e.Message;
                if (e is RpcException rpcException)
                {
                    message = "grpc status code: " + rpcException.Status.StatusCode;
                }

                statusIndicator.SetStatus("Cannot connect to server: " + message, StatusIndicator.MessageType.Error);
                tableController = null;
                searchFieldController = null;
            }
        }
    }
}