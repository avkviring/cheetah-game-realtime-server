using System;
using AOT;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class EventCommandRouter : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        private static EventCommandRouter current;
        private CheetahClient client;
        internal event EventFFI.Listener NewEventListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            EventFFI.SetListener(client.Id, OnEvent);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(EventFFI.Listener))]
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