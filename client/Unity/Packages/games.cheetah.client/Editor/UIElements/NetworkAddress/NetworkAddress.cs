using System;
using UnityEditor;
using UnityEngine.UIElements;

namespace Games.Cheetah.Client.Editor.UIElements.NetworkAddress
{
    /// <summary>
    /// Элемент для редактирования сетевого адреса (host/port) 
    /// </summary>
    public class NetworkAddress : VisualElement
    {
        private readonly TextField address;
        private readonly Button connectButton;

        public new class UxmlFactory : UxmlFactory<NetworkAddress, NetworkAddressElementUxmlTraits>
        {
        }

        public class NetworkAddressElementUxmlTraits : UxmlTraits
        {
            private readonly UxmlStringAttributeDescription storeKey = new() { name = "store-key", defaultValue = "store-key" };

            private readonly UxmlStringAttributeDescription defaultHost = new() { name = "default-address", defaultValue = "127.0.0.1:5000" };

            public override void Init(VisualElement ve, IUxmlAttributes bag, CreationContext cc)
            {
                base.Init(ve, bag, cc);
                var element = (NetworkAddress)ve;
                element.BindData(storeKey.GetValueFromBag(bag, cc), defaultHost.GetValueFromBag(bag, cc));
            }
        }


        public NetworkAddress()
        {
            VisualTreeAsset uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(
                    "Packages/games.cheetah.client/Editor/UIElements/NetworkAddress/NetworkAddress.uxml");
            uiAsset.CloneTree(this);
            address = this.Q<TextField>("address")!;
            connectButton = this.Q<Button>("connect-button")!;
        }


        private void BindData(string key, string defaultHost)
        {
            address.RegisterCallback<ChangeEvent<string>>(e => { EditorPrefs.SetString(key, e.newValue); });
            address.value = EditorPrefs.GetString(key, defaultHost);
        }


        public void AddConnectCallback(Action<string> action)
        {
            var address = this.Q<TextField>("address").value.Trim();
            if (!address.StartsWith("http"))
            {
                address = "http://" + address;
            }

            connectButton.RegisterCallback<ClickEvent>(_ => action.Invoke(address));
        }
    }
}