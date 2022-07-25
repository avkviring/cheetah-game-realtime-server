using System;
using AOT;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI
{
    public class DoubleCommandRouter : Plugin
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
        private static void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, double value)
        {
            try
            {
                current.ChangeListener?.Invoke(commandCreator, ref objectId, fieldId, value);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}