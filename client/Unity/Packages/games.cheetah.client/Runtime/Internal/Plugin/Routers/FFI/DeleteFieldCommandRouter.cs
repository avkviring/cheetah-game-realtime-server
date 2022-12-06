using System;
using AOT;
using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.FFI
{
    public class DeleteFieldCommandRouter : global::Games.Cheetah.Client.Internal.Plugin.Plugin
    {
        private static DeleteFieldCommandRouter current;
        private CheetahClient client;
        internal event FieldFFI.Listener DeleteListener;

        public void Init(CheetahClient client)
        {
            this.client = client;
            client.BeforeUpdateHook += BeforeUpdate;
            FieldFFI.SetListener(client.Id, OnChange);
        }

        private void BeforeUpdate()
        {
            current = this;
        }

        [MonoPInvokeCallback(typeof(FieldFFI.Listener))]
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