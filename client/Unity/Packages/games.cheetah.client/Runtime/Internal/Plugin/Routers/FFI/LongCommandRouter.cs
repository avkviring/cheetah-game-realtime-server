using System;
using AOT;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class LongCommandRouter : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        private static LongCommandRouter current;
        private CheetahClient client;
        internal event LongFFI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            LongFFI.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(LongFFI.Listener))]
        private static void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, long value)
        {
            try
            {
                current.ChangeListener?.Invoke(commandCreator, in objectId, fieldId, value);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}