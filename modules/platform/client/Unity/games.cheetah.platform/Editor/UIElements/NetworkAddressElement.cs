using System.Collections.Generic;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine.UIElements;

namespace Cheetah.Platform.Editor.UIElements
{
    /// <summary>
    /// Элемент для редактирования сетевого адреса (host/port) 
    /// </summary>
    public class NetworkAddressElement : VisualElement
    {
        private TextField hostField;
        private IntegerField portField;

        public string Host => this.Q<TextField>("host").value;

        public int Port => this.Q<IntegerField>("port").value;

        public new class UxmlFactory : UxmlFactory<NetworkAddressElement, NetworkAddressElementUxmlTraits>
        {
        }

        public class NetworkAddressElementUxmlTraits : UxmlTraits
        {
            UxmlStringAttributeDescription title = new UxmlStringAttributeDescription {name = "title", defaultValue = "title"};
            UxmlStringAttributeDescription id = new UxmlStringAttributeDescription {name = "store-key", defaultValue = "store-key"};
            UxmlStringAttributeDescription defaultHost = new UxmlStringAttributeDescription {name = "default-host", defaultValue = "127.0.0.1"};
            UxmlIntAttributeDescription defaultPort = new UxmlIntAttributeDescription {name = "default-port", defaultValue = 8080};
            
            public override void Init(VisualElement ve, IUxmlAttributes bag, CreationContext cc)
            {
                base.Init(ve, bag, cc);
                var element = ve as NetworkAddressElement;
                element.Q<Label>("title").text = title.GetValueFromBag(bag, cc);
                element.BindData(id.GetValueFromBag(bag, cc), defaultHost.GetValueFromBag(bag, cc), defaultPort.GetValueFromBag(bag, cc));
            }
        }


        public NetworkAddressElement()
        {
            VisualTreeAsset uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.cheetah.platform/Editor/UIElements/NetworkAddressElement.uxml");
            uiAsset.CloneTree(this);
            hostField = this.Q<TextField>("host");
            portField = this.Q<IntegerField>("port");
        }


        private void BindData(string prefix, string defaultHost, int defaultPort)
        {
            var hostPrefsKey = prefix + ".host";
            hostField.RegisterCallback<ChangeEvent<string>>(e => { EditorPrefs.SetString(hostPrefsKey, e.newValue); });
            hostField.value = EditorPrefs.GetString(hostPrefsKey, defaultHost);

            var portPrefsKey = prefix + ".port";
            portField.RegisterCallback<ChangeEvent<int>>(e => { EditorPrefs.SetInt(portPrefsKey, e.newValue); });
            portField.value = EditorPrefs.GetInt(portPrefsKey, defaultPort);
        }
    }
}