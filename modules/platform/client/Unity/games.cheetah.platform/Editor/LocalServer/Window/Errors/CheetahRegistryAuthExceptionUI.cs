using Cheetah.Platform.Editor.LocalServer.CheetahRegistry;
using UnityEditor;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.Window.Errors
{
    public class CheetahDockerRegistryAuthExceptionUI : VisualElement
    {
        public CheetahDockerRegistryAuthExceptionUI()
        {
            var uiAsset = AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(
                "Packages/games.cheetah.platform/Editor/LocalServer/Window/Errors/CheetahRegistryAuthExceptionUI.uxml");
            uiAsset.CloneTree(this);

            var dialog = new CheetahRegistrySettingsDialog();
            this.Q<VisualElement>("panel").Add(dialog);
            dialog.OnSuccessEvent += () => RemoveFromHierarchy();
        }
    }
}