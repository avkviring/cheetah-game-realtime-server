using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.LocalServer.Window.Errors
{
    public class DockerSetupDialog : VisualElement
    {
        public DockerSetupDialog()
        {
            var uiAsset = AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(
                "Packages/games.cheetah.platform/Editor/LocalServer/Window/Errors/DockerSetupError.uxml");
            uiAsset.CloneTree(this);
            this.Q<Button>("install_docker").RegisterCallback<ClickEvent>(e =>
            {
                Application.OpenURL("https://www.docker.com/products/docker-desktop");
                RemoveFromHierarchy();
            });
        }
    }
}