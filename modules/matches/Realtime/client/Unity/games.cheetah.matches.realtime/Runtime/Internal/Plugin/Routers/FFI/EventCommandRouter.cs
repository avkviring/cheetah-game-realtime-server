using System;
using AOT;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI
{
    public class EventCommandRouter : Plugin
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
        private static void OnEvent(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            try
            {
                current.NewEventListener?.Invoke(commandCreator, ref objectId, fieldId, ref data);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}