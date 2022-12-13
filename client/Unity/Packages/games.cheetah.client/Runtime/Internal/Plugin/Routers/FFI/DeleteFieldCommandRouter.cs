using System;
using AOT;
using Games.Cheetah.Client.ServerAPI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class DeleteFieldCommandRouter : Plugin
    {
        private static DeleteFieldCommandRouter current;
        private CheetahClient client;
        internal event IFieldServerAPI.Listener DeleteListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            client.serverAPI.Field.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(IFieldServerAPI.Listener))]
        private static void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, FieldType fieldType)
        {
            try
            {
                current.DeleteListener?.Invoke(commandCreator, in objectId, fieldId, fieldType);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}