using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Games.Cheetah.Client.Editor.UIElements.RoomsSelector.Provider;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine.UIElements;

namespace Games.Cheetah.Client.Editor.UIElements.RoomsSelector
{
    public class RoomsSelector : VisualElement
    {
        public class RoomsSelectorUxmlTraits : UxmlTraits
        {
            private readonly UxmlStringAttributeDescription storageKey = new UxmlStringAttributeDescription { name = "storageKey" };

            public override void Init(VisualElement ve, IUxmlAttributes bag, CreationContext cc)
            {
                base.Init(ve, bag, cc);
                var element = ve as RoomsSelector;
                element.SetStorageKey(storageKey.GetValueFromBag(bag, cc));
            }
        }

        public new class UxmlFactory : UxmlFactory<RoomsSelector, RoomsSelectorUxmlTraits>
        {
        }

        private RoomsProvider provider;
        private IList<ulong> currentRooms = new List<ulong>();
        private readonly ToolbarMenu menu;
        private bool firstUpdate = true;
        private string storageKey;
        private ulong selectedRoom;
        public event Func<ulong, Task> RoomSelectEvent;
        public event Action RoomUnselectEvent;

        public RoomsSelector()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>(
                    "Packages/games.cheetah.client/Editor/UIElements/RoomsSelector/RoomsSelector.uxml");
            uiAsset.CloneTree(this);

            menu = new ToolbarMenu();
            menu.AddToClassList("selector");
            SetDefaultMenuText();
            Insert(0, menu);
            SetEnabled(false);
        }

        private void SetDefaultMenuText()
        {
            menu.text = "Select room";
        }

        private async void OnRoomSelect(ulong room)
        {
            selectedRoom = room;
            menu.text = FormatItemCallback(room);
            EditorPrefs.SetString(storageKey, room.ToString());
            await RoomSelectEvent(room);
        }

        private static string FormatItemCallback(ulong item)
        {
            return "Room " + item;
        }


        public async Task Update()
        {
            if (provider == null)
            {
                menu.menu.MenuItems().Clear();
                return;
            } 
            var rooms = await provider.GetRooms();
            if (!rooms.SequenceEqual(currentRooms))
            {
                menu.menu.MenuItems().Clear();
                foreach (var room in rooms)
                {
                    menu.menu.AppendAction(FormatItemCallback(room), action => { OnRoomSelect(room); });
                }

                if (!rooms.Contains(selectedRoom))
                {
                    SetDefaultMenuText();
                    OnRoomReset();
                }

                currentRooms = rooms;
            }

            if (firstUpdate)
            {

                var storedRoomId = EditorPrefs.HasKey(storageKey) ? ulong.Parse(EditorPrefs.GetString(storageKey)) : 0;
                if (currentRooms.Contains(storedRoomId))
                {
                    OnRoomSelect(storedRoomId);
                }

                firstUpdate = false;
            }
        }

        private void OnRoomReset()
        {
            RoomUnselectEvent.Invoke();
        }

        public new void SetEnabled(bool enabled)
        {
            base.SetEnabled(enabled);
            menu.SetEnabled(enabled);
            
        }

        public void SetProvider(RoomsProvider provider)
        {
            SetEnabled(true);
            this.provider = provider;
        }

        private void SetStorageKey(string storageKey)
        {
            this.storageKey = storageKey;
        }

        public void RemoveProvider()
        {
            provider = null;
            SetEnabled(false);
        }
        
        
        
    }
}