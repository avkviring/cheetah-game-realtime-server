using System;
using System.IO;
using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Android.Editor
{
    public class GooglePlayGamesSettings
    {
        private static readonly string PATH = Application.dataPath + "/../ProjectSettings/GooglePlayGamesSettings.json";
        public string AppId = "";


        public static GooglePlayGamesSettings GetOrCreateSettings()
        {
            try
            {
                var content = File.ReadAllText(PATH);
                return JsonUtility.FromJson<GooglePlayGamesSettings>(content);
            }
            catch (Exception e)
            {
                return new GooglePlayGamesSettings();
            }
        }

        internal void Save()
        {
            File.WriteAllText(PATH, JsonUtility.ToJson(this));
        }
    }

    public class GooglePlayGamesSettingsProvider : SettingsProvider
    {
        private GooglePlayGamesSettings settings;
        public const string PATH = "Project/Cheetah/Google PlayGames";

        class Styles
        {
            public static GUIContent appId = new GUIContent("Android APP_ID");
        }

        public GooglePlayGamesSettingsProvider(string path, SettingsScope scope)
            : base(path, scope)
        {
        }


        public override void OnActivate(string searchContext, VisualElement rootElement)
        {
            settings = GooglePlayGamesSettings.GetOrCreateSettings();
        }

        public override void OnGUI(string searchContext)
        {
            var rect = new Rect(10, 10, 300, 40);
            GUILayout.BeginArea(rect);
            settings.AppId = EditorGUILayout.TextField("Android APP_ID", settings.AppId);
            GUILayout.EndArea();
        }

        public override void OnDeactivate()
        {
            settings?.Save();
        }

        // Register the SettingsProvider
        [SettingsProvider]
        public static SettingsProvider CreateMyCustomSettingsProvider()
        {
            var provider = new GooglePlayGamesSettingsProvider(PATH, SettingsScope.Project);
            provider.keywords = GetSearchKeywordsFromGUIContentProperties<Styles>();
            return provider;
        }
    }
}