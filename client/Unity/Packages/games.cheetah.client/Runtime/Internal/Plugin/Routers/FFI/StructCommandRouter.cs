using System;
using AOT;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class StructCommandRouter : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        private static StructCommandRouter current;
        private CheetahClient client;
        internal event StructureFFI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            StructureFFI.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(StructureFFI.Listener))]
        private static void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer data)
        {
            try
            {
                current.ChangeListener?.Invoke(commandCreator, in objectId, fieldId, ref data);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}