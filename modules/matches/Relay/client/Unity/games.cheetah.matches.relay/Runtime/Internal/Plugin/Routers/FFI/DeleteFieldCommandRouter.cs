using System;
using AOT;
using Cheetah.Matches.Relay.Internal.FFI;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal.Plugin.Routers.FFI
{
    public class DeleteFieldCommandRouter : Plugin
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
        private static void OnChange(ushort commandCreator, ref CheetahObjectId objectId, ushort fieldId, FieldType fieldType)
        {
            try
            {
                current.DeleteListener?.Invoke(commandCreator, ref objectId, fieldId, fieldType);
            }
            catch (Exception e)
            {
                current.client.OnException(e);
            }
        }
    }
}