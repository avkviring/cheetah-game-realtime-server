using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class StructCommandRouter : Plugin
    {
        private static StructCommandRouter current;
        private CheetahClient client;
        internal event IStructureServerAPI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            client.serverAPI.Structure.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(IStructureServerAPI.Listener))]
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