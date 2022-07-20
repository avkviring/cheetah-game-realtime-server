using System.IO;
using System.Text;
using System.Threading.Tasks;
using Cheetah.Matches.Factory.Editor.Configurations;
using Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.Provider;
using Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.UI;
using Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.UI.Controller;
using Cheetah.Matches.Relay.Editor.UIElements.HistoryTextField;
using Cheetah.Matches.Relay.Editor.UIElements.RoomsSelector;
using Cheetah.Matches.Relay.Editor.UIElements.RoomsSelector.Provider;
using Cheetah.Matches.Relay.Editor.UIElements.StatusIndicator;
using Cheetah.Matches.Relay.Editor.UIElements.Table;
using Cheetah.Platform.Editor.Connector;
using Grpc.Core;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Relay.Editor.NetworkCommandsViewer
{
    /// <summary>
    /// Панель просмотра сетевых команд с сервера
    /// </summary>
    public class NetworkCommandsViewerWindow : EditorWindow
    {
        /// <summary>
        /// Удаленные провайдеры данных или тестовые
        /// </summary>
        private const bool RemoteProviders = true;

        private const string PathToStyleSheet = "Packages/games.cheetah.matches.relay/Editor/NetworkCommandsViewer/UI/Styles.uss";
        private const string PathToUxml = "Packages/games.cheetah.matches.relay/Editor/NetworkCommandsViewer/UI/Window.uxml";
        private const int UpdateTime = 60 / 2;
        private const int ErrorUpdateTime = 60;

        private readonly Columns columns = new Columns();
        private TableController tableController;
        private StatusIndicator statusIndicator;
        private TracedCommandsProvider provider;
        private ConfigurationsProvider configurationsProvider;
        private RoomsProvider roomsProvider;
        private RoomsSelector roomsSelector;
        private int timeToUpdate;
        private bool pause;
        private SearchFieldController searchFieldController;
        private ToolbarButton pauseButton;
        private bool connectError;

        [MenuItem("Window/Cheetah/Relay commands viewer")]
        public static void ShowServerCommandsViewerWindow()
        {
            var window = GetWindow<NetworkCommandsViewerWindow>();
            window.titleContent = new GUIContent("Relay commands viewer");
        }

        public void OnEnable()
        {
            provider = RemoteProviders
                ? (TracedCommandsProvider)new RemoteTracedCommandsProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestTracedCommandsProvider();
            inUpdate = false; // иначе он может остаться в true после перезагрузки домена
            LoadStyles();
            var ui = LoadVisualTree();
            new ColumnsMenuController(ui.Q<ToolbarMenu>("colummenu"), columns);
            statusIndicator = ui.Q<StatusIndicator>("status");
            searchFieldController = new SearchFieldController(ui.Q<HistoryTextField>("searchField"), statusIndicator, provider);
            ConfigureRoomSelector(ui);
            InitializeTable(ui);
            rootVisualElement.Add(ui);
            SetupClearButton(ui);
            SetupPauseButton(ui);
            SetupHelpButton(ui);
            SetupExportButton(ui);
        }
        
        private void ConfigureRoomSelector(TemplateContainer ui)
        {
            roomsProvider = RemoteProviders
                ? (RoomsProvider)new RemoteRoomsProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestRoomsProvider();
            configurationsProvider = RemoteProviders
                ? (ConfigurationsProvider)new RemoteConfigurationsProvider(LocalClusterConnectorFactory.CreateConnector())
                : new TestConfigurationsProvider();
            roomsSelector = ui.Q<RoomsSelector>("room-selector");
            roomsSelector.SetProvider(roomsProvider);
            roomsSelector.RoomSelectEvent += provider.SetRoom;
            roomsSelector.RoomSelectEvent += UpdateConfigurationProvider;
            roomsSelector.RoomUnselectEvent += provider.ResetRooms;
        }

        private Task UpdateConfigurationProvider(ulong arg)
        {
            return configurationsProvider.Load();
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
        
        private void SetupExportButton(TemplateContainer ui)
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
                    builder.Append(command.HasFieldId ? command.FieldId :"None").Append("\t");
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

        private void InitializeTable(VisualElement ui)
        {
            var table = ui.Q<TableElement>("commands-table");
            tableController = new TableController(table, columns, provider,configurationsProvider);
        }


        private bool inUpdate;
        private ToolbarButton clearButton;


        private async void Update()
        {
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
            searchFieldController.Enabled(provider.IsReady() && !pause && !connectError);
            pauseButton.SetEnabled(!connectError);

            var connected = !pause && !connectError;
            roomsSelector.SetEnabled(connected);
            clearButton.SetEnabled(connected);
            pauseButton.text = pause ? "Resume" : "Pause";
        }


        private async void OnDestroy()
        {
            await roomsProvider.Destroy();
            await provider.Destroy();
        }
    }
}