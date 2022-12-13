using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class DoubleCommandRouter : Plugin
    {
        private static DoubleCommandRouter current;
        private CheetahClient client;
        internal event IDoubleServerAPI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            client.serverAPI.Double.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(IDoubleServerAPI.Listener))]
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