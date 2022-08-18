using System;
using System.IO;
using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.Configuration
{
    public class ConfigurationSettings
    {
        private static readonly string PATH = Application.dataPath + "/../ProjectSettings/CheetahPlatformConfigurationSettings.json";
        public string Directory = "Assets/Editor/Cheetah/Production/";


        public static ConfigurationSettings GetOrCreateSettings()
        {
            try
            {
                var content = File.ReadAllText(PATH);
                return JsonUtility.FromJson<ConfigurationSettings>(content);
            }
            catch (Exception)
            {
                return new ConfigurationSettings();
            }
        }

        internal void Save()
        {
            File.WriteAllText(PATH, JsonUtility.ToJson(this));
        }
    }

    public class ConfigurationSettingsProvider : SettingsProvider
    {
        private ConfigurationSettings settings;
        private const string Path = "Project/Cheetah/Directories";

        public ConfigurationSettingsProvider(string path, SettingsScope scope)
            : base(path, scope)
        {
        }


        public override void OnActivate(string searchContext, VisualElement rootElement)
        {
            settings = ConfigurationSettings.GetOrCreateSettings();
        }

        public override void OnGUI(string searchContext)
        {
            var rect = new Rect(10, 10, 500, 40);
            GUILayout.BeginArea(rect);
            settings.Directory = EditorGUILayout.TextField("Configuration directory", settings.Directory);
            GUILayout.EndArea();
        }

        public override void OnDeactivate()
        {
            settings?.Save();
        }

        [SettingsProvider]
        public static SettingsProvider CreateSettingsProvider()
        {
            return new ConfigurationSettingsProvider(Path, SettingsScope.Project);
        }
    }
}