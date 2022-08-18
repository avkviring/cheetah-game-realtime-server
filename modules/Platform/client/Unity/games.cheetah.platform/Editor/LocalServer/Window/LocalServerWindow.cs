using System;
using System.Collections.Generic;
using Cheetah.Platform.Editor.LocalServer.Docker;
using Cheetah.Platform.Editor.LocalServer.SharedConfig;
using Cheetah.Platform.Editor.LocalServer.Window.Errors;
using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.Window
{
    /// <summary>
    ///     Окно для конфигурирования и запуска серверных приложений
    /// </summary>
    public class LocalServerWindow : EditorWindow, IDockerProgressListener
    {
        private PlatformInDockerRunner platformRunner;
        private VisualElement controlPanelVisualElement;
        private VisualElement errorPanel;
        private ProgressBar progressBar;
        private Button restartButton;
        private Button stopButton;
        readonly SystemApplicationsConfigurator systemConfigurator = new();
        

        private void OnDestroy()
        {
            platformRunner.OnStatusChange -= UpdateStatus;
        }

        private void CreateGUI()
        {
            try
            {
                var uiAsset =
                    AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/LocalServer/Window/UI.uxml");
                rootVisualElement.Clear();
                rootVisualElement.Add(uiAsset.Instantiate());
                SetupControlPanel();
                
            }
            catch (Exception e)
            {
                Debug.LogException(e);
                Close();
            }
        }

        public void SetProgressTitle(string title)
        {
            progressBar.title = title;
        }

        public void SetProgress(int percent)
        {
            progressBar.value = percent;
        }

        [MenuItem("Window/Cheetah/Local Server")]
        public static void ShowWindow()
        {
            GetWindow(typeof(LocalServerWindow), false, "Cheetah Local Server");
        }

        private void SetupControlPanel()
        {
            controlPanelVisualElement = rootVisualElement.Q<VisualElement>("ControlPanel");
            restartButton = controlPanelVisualElement.Q<Button>("start");
            restartButton.RegisterCallback<ClickEvent>(OnStartClick);
            stopButton = controlPanelVisualElement.Q<Button>("stop");
            stopButton.RegisterCallback<ClickEvent>(OnStopClick);
            progressBar = controlPanelVisualElement.Q<ProgressBar>("progress");
            errorPanel = controlPanelVisualElement.Q<VisualElement>("error");

            var content = rootVisualElement.Q<ScrollView>("content");
            var configurations = new List<IApplicationsConfigurator>();
            configurations.Add(systemConfigurator);
            configurations.AddRange(Registry.GetConfigurators());
            foreach (var configurator in configurations)
            {
                var foldout =
                    AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/LocalServer/Window/LocalServerWindow.uxml")
                        .Instantiate();
                foldout.Q<Label>().text = configurator.Title;
                content.Add(foldout);
                foldout.Q<VisualElement>("foldout")
                    .Q<VisualElement>("unity-content")
                    .Add(configurator.CreateUI());
            }
        }

        private async void OnStartClick(ClickEvent evt)
        {
            platformRunner = new PlatformInDockerRunner(systemConfigurator);
            platformRunner.OnStatusChange += UpdateStatus;
            errorPanel.Clear();
            try
            {
                await platformRunner.Restart(this);
            }
            catch (Exception e)
            {
                ShowException(e);
            }
        }

        private async void OnStopClick(ClickEvent evt)
        {
            try
            {
                await platformRunner.Stop(this);
            }
            catch (Exception e)
            {
                ShowException(e);
            }
        }


        private void UpdateStatus(Status status)
        {
            foreach (var uiConfiguration in Registry.GetConfigurators()) uiConfiguration.OnUpdateStatus(status);

            switch (status)
            {
                case Status.Unknown:
                    restartButton.text = "Start";
                    stopButton.text = "Stopped";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(false);
                    progressBar.value = 100;
                    progressBar.title = "Unknown";
                    break;
                case Status.Disconnected:
                    restartButton.text = "Start";
                    stopButton.text = "Stopped";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(true);
                    progressBar.value = 100;
                    progressBar.title = "Cannot connect to docker.";
                    break;
                case Status.Started:
                    restartButton.text = "Restart";
                    stopButton.text = "Stop";
                    stopButton.SetEnabled(true);
                    restartButton.SetEnabled(true);
                    progressBar.value = 100;
                    progressBar.title = "started";
                    break;
                case Status.Stopped:
                    restartButton.text = "Start";
                    stopButton.text = "Stopped";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(true);
                    progressBar.value = 100;
                    progressBar.title = "stopped";
                    break;
                case Status.Fail:
                    restartButton.text = "Start";
                    stopButton.text = "Stopped";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(true);
                    progressBar.value = 0;
                    progressBar.title = "Starting fail. See logs in unity console.";
                    break;
                case Status.Starting:
                    restartButton.text = "Starting";
                    stopButton.text = "Stop";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(false);
                    break;
                case Status.Stopping:
                    restartButton.text = "Start";
                    stopButton.text = "Stopping";
                    stopButton.SetEnabled(false);
                    restartButton.SetEnabled(false);
                    break;
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }


        private void ShowException(Exception e)
        {
            Debug.LogException(e);
            if (e is DockerConnectException)
            {
                errorPanel.Add(new DockerSetupDialog());
            }
        }
    }
}