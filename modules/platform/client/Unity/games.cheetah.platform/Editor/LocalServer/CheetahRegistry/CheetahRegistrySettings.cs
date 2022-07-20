using System.IO;
using System.Text;
using JetBrains.Annotations;
using UnityEditor;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer.CheetahRegistry
{
    /// <summary>
    ///     Хранение Login/Password для доступа к https://docker.registry.cheetah.games
    /// </summary>
    public class CheetahRegistrySettings
    {
        public virtual string Login { get; set; }
        public virtual string Password { get; set; }
    }

    public class CheetahRegistrySettingsFromPrefs : CheetahRegistrySettings
    {
        private const string LoginKey = "cheetah.platform.auth.name";
        private const string PasswordKey = "cheetah.platform.auth.password";
        public static CheetahRegistrySettings Instance = new CheetahRegistrySettingsFromPrefs();

        public override string Login
        {
            get => EditorPrefs.GetString(LoginKey, "");

            set => EditorPrefs.SetString(LoginKey, value);
        }

        public override string Password
        {
            get => EditorPrefs.GetString(PasswordKey, "");
            set => EditorPrefs.SetString(PasswordKey, value);
        }
    }

    public class CheetahRegistrySettingsFromConfig : CheetahRegistrySettings
    {
        [CanBeNull]
        public static CheetahRegistrySettingsFromConfig Load()
        {
            var path = Path.Combine(Application.dataPath, "../cheetah-docker-registry.json");
            if (!File.Exists(path)) return null;
            var content = Encoding.Default.GetString(File.ReadAllBytes(path));
            var result = JsonUtility.FromJson<Config>(content);
            return new CheetahRegistrySettingsFromConfig
            {
                Login = result.Login,
                Password = result.Password
            };
        }

        private class Config
        {
            public string Login;
            public string Password;
        }
    }
}