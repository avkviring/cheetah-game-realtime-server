using System;
using AOT;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class DoubleCommandRouter : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        private static DoubleCommandRouter current;
        private CheetahClient client;
        internal event DoubleFFI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            DoubleFFI.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(DoubleFFI.Listener))]
        private static void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, double value)
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