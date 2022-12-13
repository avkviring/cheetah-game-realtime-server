using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;
using UnityEngine;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class LongCommandRouter : Plugin
    {
        private static LongCommandRouter current;
        private CheetahClient client;
        internal event ILongServerAPI.Listener ChangeListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            client.serverAPI.Long.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(ILongServerAPI.Listener))]
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