using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.CheetahRegistry
{
    public class CheetahRegistrySettingsProvider : SettingsProvider
    {
        private const string Path = "Preferences/Cheetah/Authentication";

        private CheetahRegistrySettingsProvider() : base(Path, SettingsScope.User)
        {
        }

        [SettingsProvider]
        public static SettingsProvider Create()
        {
            return new CheetahRegistrySettingsProvider();
        }


        public override void OnActivate(string searchContext, VisualElement rootElement)
        {
            rootElement.Add(new CheetahRegistrySettingsDialog());
            base.OnActivate(searchContext, rootElement);
        }
    }
}