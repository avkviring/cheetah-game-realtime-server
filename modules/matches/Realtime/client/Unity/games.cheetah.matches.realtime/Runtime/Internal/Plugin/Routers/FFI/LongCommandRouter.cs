using System;
using AOT;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.Plugin.Routers.FFI
{
    public class LongCommandRouter : Plugin
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
        private static void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, long value)
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