using UnityEditor;
using UnityEngine.UIElements;

namespace Games.Cheetah.Client.Editor.UIElements.StatusIndicator
{
    public class StatusIndicator : VisualElement
    {
        public class StatusIndicatorUxmlTraits : UxmlTraits
        {
        }

        public new class UxmlFactory : UxmlFactory<StatusIndicator, StatusIndicatorUxmlTraits>
        {
        }


        public StatusIndicator()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.client/Editor/UIElements/StatusIndicator/StatusIndicator.uxml");
            uiAsset.CloneTree(this);
        }
        public enum MessageType
        {
            Regular,
            Warning,
            Error
        }

        public void SetStatus(string text, MessageType type)
        {
            var label = GetStatusLabel();
            label.ClearClassList();
            label.AddToClassList("status");
            var style = type switch
            {
                MessageType.Regular => "regular",
                MessageType.Warning => "warning",
                MessageType.Error => "error",
                _ => ""
            };
            style += EditorGUIUtility.isProSkin ? "-dark" : "-light";
            label.AddToClassList(style);
            label.text = text;
        }

        public void ResetStatus()
        {
            var label = GetStatusLabel();
            label.text = "";
        }

        private Label GetStatusLabel()
        {
            return this.Q<Label>("status");
        }
    }
}