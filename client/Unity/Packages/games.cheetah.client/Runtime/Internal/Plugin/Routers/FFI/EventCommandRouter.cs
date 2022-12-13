using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class EventCommandRouter : Plugin
    {
        private static EventCommandRouter current;
        private CheetahClient client;
        internal event IEventServerAPI.Listener NewEventListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            client.serverAPI.Event.SetListener(client.Id, OnEvent);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(IEventServerAPI.Listener))]
        private static void OnEvent(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            try
            {
                current.NewEventListener?.Invoke(commandCreator, in objectId, fieldId, ref data);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}